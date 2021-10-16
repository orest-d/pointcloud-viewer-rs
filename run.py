from flask import Flask, make_response

app = Flask(__name__)

@app.route("/")
@app.route("/index.html")
def index():
    html = open("assets/index.html").read()
    return html

@app.route("/assets/<name>")
def wasm(name):

    r = make_response(open(f"assets/{name}","rb").read())
    if name.endswith(".wasm"):
        r.headers.set('Content-Type', "application/wasm")
    return r

@app.route("/data.csv")
def csv():
    print("GET CSV")
    html = open("data.csv").read()
    return html

if __name__ == "__main__":
    app.run(debug=True,port=8080)
