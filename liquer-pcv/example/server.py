# Make it run from the examples directory
import sys
sys.path.append("..")

from liquer import *
import pandas as pd
import numpy as np
import liquer.ext.lq_pandas # Add pandas support to liquer so that the dataframe conversions work
import liquer_pcv # Register Pointcloud viewer

### Create Flask app and register LiQuer blueprint
from flask import Flask
import liquer.server.blueprint as bp
app = Flask(__name__)

url_prefix='/liquer'
app.register_blueprint(bp.app, url_prefix=url_prefix)


@first_command
def hello():
    return "Hello"

@app.route('/')
@app.route('/index.html')
def index():
    return """<h1>Hello-world app</h1>
    <ul>
    <li><a href="/liquer/q/hello">just hello</a></li>
    <li><a href="/liquer/q/harmonic/harmonic.html">harmonic 100</a></li>
    <li><a href="/liquer/q/harmonic-100000/noise-0.1/pointcloud-viewer.html">noisy harmonic 100k rows</a></li>
    <li><a href="/liquer/q/harmonic-1000000/noise-0.2/pointcloud-viewer.html">noisy harmonic 1M rows (big!)</a></li>
    </ul>
    """

@first_command
def harmonic(n=100):
    a = np.linspace(0,2*np.pi,n)
    segment = np.array(a*10/(2*np.pi),dtype=int)
    return pd.DataFrame(
        dict(
            a=a,
            x=np.sin(a),
            y=np.cos(a),
            x2=np.sin(2*a),
            y2=np.cos(2*a),
            x3=np.sin(3*a),
            y3=np.cos(3*a),
            x4=np.sin(4*a),
            y4=np.cos(4*a),
            segment=segment,
            label=[f"{i+1}/{n}" for i in range(n)]
        )
    )

@command
def noise(df, sigma=0.1):
    columns = [c for c in df.columns if c.startswith("x") or c.startswith("y")]
    for c in columns:
        noise = np.random.normal(0.0,sigma,len(df))
        df[c]+=noise
    return df

if __name__ == '__main__':
    app.run()
