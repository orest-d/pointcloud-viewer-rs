from io import StringIO, BytesIO
from urllib.request import urlopen
import pandas as pd
import numpy as np
from liquer.state_types import StateType, register_state_type, mimetype_from_extension
from liquer.commands import command, first_command
from liquer.parser import encode, decode
from liquer.query import evaluate

class DataframeStateType(StateType):
    def identifier(self):
        return "dataframe"

    def default_extension(self):
        return "csv"

    def is_type_of(self, data):
        return isinstance(data, pd.DataFrame)

    def as_bytes(self, data, extension=None):
        if extension is None:
            extension = self.default_extension()
        assert self.is_type_of(data)
        mimetype = mimetype_from_extension(extension)
        if extension == "csv":
            output = StringIO()
            data.to_csv(output, index=False)
            return output.getvalue().encode("utf-8"), mimetype
        elif extension == "tsv":
            output = StringIO()
            data.to_csv(output, index=False, sep="\t")
            return output.getvalue().encode("utf-8"), mimetype
        elif extension == "json":
            output = StringIO()
            data.to_json(output, index=False, orient="table")
            return output.getvalue().encode("utf-8"), mimetype
        elif extension in ("html", "htm"):
            output = StringIO()
            data.to_html(output, index=False)
            return output.getvalue().encode("utf-8"), mimetype
        elif extension == "xlsx":
            output = BytesIO()
            writer = pd.ExcelWriter(output, engine='xlsxwriter')
            data.to_excel(writer)
            writer.close()
            return output.getvalue()
        elif extension == "msgpack":
            output = BytesIO()
            data.to_msgpack(output)
            return output.getvalue()
        else:
            raise Exception(
                f"Serialization: file extension {extension} is not supported by dataframe type.")

    def from_bytes(self, b: bytes, extension=None):
        if extension is None:
            extension = self.default_extension()
        f = BytesIO()
        f.write(b)
        f.seek(0)

        if extension == "csv":
            return pd.read_csv(f)
        elif extension == "tsv":
            return pd.read_csv(f, sep="\t")
        elif extension == "json":
            return pd.read_json(f)
        elif extension == "xlsx":
            return pd.read_excel(f)
        elif extension == "msgpack":
            return pd.read_msgpack(f)
        raise Exception(
            f"Deserialization: file extension {extension} is not supported by dataframe type.")

    def copy(self, data):
        return data.copy()


DATAFRAME_STATE_TYPE = DataframeStateType()
register_state_type(pd.DataFrame, DATAFRAME_STATE_TYPE)


@first_command
def df_from(url, extension=None):
    """Load data from URL
    """
    if extension is None:
        extension = url.split(".")[-1]
        if extension not in "csv tsv xls xlsx msgpack".split():
            extension = "csv"
    if url.startswith("http:") or url.startswith("https:") or url.startswith("ftp:"):
        f = BytesIO(urlopen(url).read())
    else:
        f = open(url)
    if extension == "csv":
        return pd.read_csv(f)
    elif extension == "tsv":
        return pd.read_csv(f, sep="\t")
    elif extension in ("xls", "xlsx"):
        return pd.read_excel(f)
    elif extension == "msgpack":
        return pd.read_msgpack(f)
    else:
        raise Exception(f"Unsupported file extension: {extension}")


@command
def append_df(df, url, extension=None):
    """Append dataframe from URL
    """
    df1 = df_from(url, extension=extension)
    return df.append(df1, ignore_index=True)


@command
def eq(state, *column_values):
    """Equals filter
    Accepts one or more column-value pairs. Keep only rows where value in the column equals specified value.
    Example: eq-column1-1
    """
    df = state.get()
    assert state.type_identifier == "dataframe"
    for i in range(0, len(column_values), 2):
        c = column_values[i]
        v = column_values[i+1]
        state.log_info(f"Equals: {c} == {v}")
        index = np.array([x == v for x in df[c]], np.bool)
        try:
            if int(v) == float(v):
                index = index | (df[c] == int(v))
            else:
                index = index | (df[c] == float(v))
        except:
            pass
        df = df.loc[index, :]
    return state.with_data(df)

@command
def teq(state, *column_values):
    """Tag-Equals filter. Expects, that a first row contains tags and/or metadata
    Tag row is ignored in comparison, but prepended to the result (in order to maintain the first row in the results).
    Accepts one or more column-value pairs. Keep only rows where value in the column equals specified value.
    Example: teq-column1-1
    """
    df = state.get()
    tags = df.iloc[:1,:]
    df = df.iloc[1:,:]
    assert state.type_identifier == "dataframe"
    for i in range(0, len(column_values), 2):
        c = column_values[i]
        v = column_values[i+1]
        state.log_info(f"Equals: {c} == {v}")
        index = np.array([x == v for x in df[c]], np.bool)
        try:
            if int(v) == float(v):
                index = index | (df[c] == int(v))
            else:
                index = index | (df[c] == float(v))
        except:
            pass
        df = df.loc[index, :]
    df = tags.append(df,ignore_index=True)
    return state.with_data(df)

@command
def qsplit_df(state, *columns):
    """Quick/query split of dataframe by columns
    Creates a dataframe with unique (combinations of) value from supplied columns and queries
    to obtain the corresponding filtered dataframes from the original dataframe.
    Resulting queries are put in query column. Name of the query column
    can be overriden by query_column state variable.
    """
    df = state.get()
    if len(columns) == 1:
        keys = [(x,) for x in sorted(df.groupby(by=list(columns)).groups.keys())]
    else:
        keys = sorted(df.groupby(by=list(columns)).groups.keys())

    query_column = state.vars.get("query_column")
    if query_column is None:
        query_column = "query"

    sdf = pd.DataFrame(columns=list(columns)+[query_column])
    data = []
    ql = decode(state.query)
    for row in keys:
        pairs = list(zip(columns, row))
        d = dict(pairs)
        query = encode(ql+[["eq"]+[str(x) for p in pairs for x in p]])
        d[query_column] = query
        sdf = sdf.append(d, ignore_index=True)

    return state.with_data(sdf)

@command
def qtsplit_df(state, *columns):
    """Quick/query split of dataframe by columns (version expecting a first row with tags)
    Creates a dataframe with unique (combinations of) value from supplied columns and queries
    to obtain the corresponding filtered dataframes from the original dataframe.
    Resulting queries are put in query column. Name of the query column
    can be overriden by query_column state variable.
    """
    df = state.get()
    tags = df.iloc[0]
    df = df.iloc[1:]

    if len(columns) == 1:
        keys = [(x,) for x in sorted(df.groupby(by=list(columns)).groups.keys())]
    else:
        keys = sorted(df.groupby(by=list(columns)).groups.keys())

    query_column = state.vars.get("query_column")
    if query_column is None:
        query_column = "query"

    sdf = pd.DataFrame(columns=list(columns)+[query_column])
    sdf = sdf.append({c:tags[c] for c in columns}, ignore_index=True)
    data = []
    ql = decode(state.query)
    for row in keys:
        pairs = list(zip(columns, row))
        d = dict(pairs)
        query = encode(ql+[["teq"]+[str(x) for p in pairs for x in p]])
        d[query_column] = query
        sdf = sdf.append(d, ignore_index=True)

    return state.with_data(sdf)

@command
def split_df(state, *columns):
    """Split of dataframe by columns
    Creates a dataframe with unique (combinations of) value from supplied columns and queries
    to obtain the corresponding filtered dataframes from the original dataframe.

    This behaves like qsplit_df, with two important differenced:
    - each generated query is evaluated (and thus eventually cached)
    - link is generated and put into link column (state variable link_column)
    The split_link_type state variable is used to determine the link type; url by default. 
    """
    state = qsplit_df(state, *columns)
    df = state.get().copy()

    query_column = state.vars.get("query_column")
    if query_column is None:
        query_column = "query"

    link_column = state.vars.get("link_column")
    if link_column is None:
        link_column = "link"

    split_link_type = state.vars.get("split_link_type")
    if split_link_type is None:
        split_link_type = "url"

    df.loc[:,link_column] = [evaluate(encode(decode(q)+[["link",split_link_type]])).get() for q in df[query_column]]
    return state.with_data(df)

@command
def tsplit_df(state, *columns):
    """Split of dataframe by columns (version of split_df expecting a first row with tags)
    """
    state = qtsplit_df(state, *columns)
    df = state.get().copy()

    query_column = state.vars.get("query_column")
    if query_column is None:
        query_column = "query"

    link_column = state.vars.get("link_column")
    if link_column is None:
        link_column = "link"

    split_link_type = state.vars.get("split_link_type")
    if split_link_type is None:
        split_link_type = "url"

    df.loc[:,link_column] = [""]+[evaluate(encode(decode(q)+[["link",split_link_type]])).get() for q in list(df[query_column])[1:]]
    return state.with_data(df)