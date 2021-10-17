import logging
from liquer import *
import pandas as pd
import numpy as np
import liquer.ext.lq_pandas
import liquer.ext.meta
import liquer.ext.basic
import liquer.ext.lq_plotly
from liquer.state import set_var
from liquer.context import Context

### Create Flask app and register LiQuer blueprint
from flask import Flask
import liquer.server.blueprint as bp
from liquer.store import get_store, set_store, MemoryStore
app = Flask(__name__)
# Setting the logger to make debugging info visible
logging.basicConfig()
logger = logging.getLogger(__name__)
logger.setLevel(logging.DEBUG)
werkzeug_logger = logging.getLogger('werkzeug')
werkzeug_logger.setLevel(logging.INFO)

# Registering the liquer blueprint under a given url prefix and letting LiQuer know where it is...
url_prefix='/liquer'
app.register_blueprint(bp.app, url_prefix=url_prefix)
set_var("api_path",url_prefix+"/q/")
set_var("server","http://localhost:5000")

set_store(MemoryStore())
get_store().store(
    "assets/pointcloud-viewer/pointcloud-viewer.wasm",
    open("../assets/pointcloud-viewer.wasm","rb").read(),
    dict(
        extension="wasm",
        mimetype="application/wasm"
        ))
for name in ["gl", "quad-url", "sapp_jsutils"]:
    get_store().store(
        f"assets/pointcloud-viewer/{name}.js",
        open(f"../assets/{name}.js","rb").read(),
        dict(
            extension="js",
            mimetype="text/javascript"
            ))


@first_command
def hello():
    return "Hello"

@app.route('/')
@app.route('/index.html')
def index():
    return """<h1>Hello-world app</h1>
    <ul>
    <li><a href="/liquer/q/hello">just hello</a></li>
    <li><a href="/liquer/q/harmonic">harmonic 100</a></li>
    <li><a href="/liquer/q/harmonic/plotly_chart-xy-x-y/xy.html">harmonic 100 plot</a></li>
    </ul>
    """

@first_command
def harmonic(n=100):
    a = np.linspace(0,2*np.pi,n)
    return pd.DataFrame(
        dict(
            a=a,
            x=np.sin(a),
            y=np.cos(a),
            x2=np.sin(2*a),
            y2=np.cos(2*a),
            label=[f"{i+1}/{n}" for i in range(n)]
        )
    )

@command
def pointcloud(state,name,context=None):
    if context is None:
        context = Context()
    
    assert name=="viewer.html"
    return state.with_filename("pointcloud-viewer.html").with_data("""
<!doctype html>
<html lang="en">

<head>
    <meta charset="utf-8">
    <title>Poincloud viewer</title>
    <style>
        html,
        body,
        canvas {
            margin: 0px;
            padding: 0px;
            width: 100%;
            height: 100%;
            overflow: hidden;
            position: absolute;
            background: black;
            z-index: 0;
        }
    </style>
</head>

<body>
    <canvas id="glcanvas" tabindex='1'></canvas>
    <script src="/liquer/api/store/data/assets/pointcloud-viewer/gl.js"></script>
    <script src="/liquer/api/store/data/assets/pointcloud-viewer/sapp_jsutils.js"></script>
    <script src="/liquer/api/store/data/assets/pointcloud-viewer/quad-url.js"></script>
    <script>load("/liquer/api/store/data/assets/pointcloud-viewer/pointcloud-viewer.wasm");</script> <!-- Your compiled wasm file -->
</body>

</html>
    """)

if __name__ == '__main__':
    app.run()
