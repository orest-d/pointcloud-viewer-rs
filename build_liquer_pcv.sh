#!/bin/bash

sh build_web.sh

cp assets/gl.js liquer-pcv/liquer_pcv/assets
cp assets/quad-url.js liquer-pcv/liquer_pcv/assets
cp assets/sapp_jsutils.js liquer-pcv/liquer_pcv/assets
cp assets/pointcloud-viewer.wasm liquer-pcv/liquer_pcv/assets
cp README.md liquer-pcv/README.md

cd liquer-pcv
python setup.py sdist bdist_wheel
cd ..