from liquer import *
from liquer.context import Context
from pathlib import Path
from liquer.store import web_mount_folder, get_store, get_web_store

def assets_path():
    return str((Path(__file__).parent/"assets").resolve())

def init():
    web_mount_folder("pointcloud-viewer", assets_path())

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
    <script src="/liquer/web/pointcloud-viewer/gl.js"></script>
    <script src="/liquer/web/pointcloud-viewer/sapp_jsutils.js"></script>
    <script src="/liquer/web/pointcloud-viewer/quad-url.js"></script>
    <script>load("/liquer/web/pointcloud-viewer/pointcloud-viewer.wasm");</script> <!-- Your compiled wasm file -->
</body>

</html>
    """)
