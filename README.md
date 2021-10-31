# Pointcloud viewer

![Screenshot](https://orest-d.github.io/pointcloud-viewer-rs/assets/demo.jpg)

Pointcloud viewer is a tool for visualization and exploratory data analysis.
It can read tabular data (i.e. a dataframe) and display selected columns in 2D.
Pointcloud viewer is designed to handle large amount of points (tested up to 2M),
where the point density is more relevant than individual points.
Point density is shown by a color gradient. To help to make points more visible (especially in smaller datasets),
points can be smeared by a Gaussian function.

See live [demo](https://orest-d.github.io/pointcloud-viewer-rs/).

# Features

* Display selected columns
* Data in the selected columns can be transformed to a different scale: linear, logarithmic or quantile.
* Display the point density via a color gradient with tunable brightness
* Zoom, move, change aspect ratio
* Show the row of data under the mouse cursor
* Optional Gaussian smearing
* Optionally specify a weight for each point
* Highlight a group of points based on a selected value in a specified column
* Highlighting supports four different modes (depending what data are shown)
* Statistics calculator (experimental)
* Pointcloud viewer can be compiled to webassembly and used on the web - either in connection to LiQuer framework or standalone. It as well can be compiled to a desktop application.


# LiQuer support

Pointcloud viewer is designed for [LiQuer](https://orest-d.github.io/liquer/)

## Install

Assuming you have a LiQuer system set up, you can add Pointcloud viewer by

```
pip install liquer-pcv
```

In the code, when importing LiQuer command modules, use

```python
import liquer_pcv
```

This will add a 'pointcloud' command, which can be used in an interractive LiQuer session
to display the dataframe. Simply finish a LiQuer query with 'pointcloud-viewer.html' and the display will show up.

See [example](https://github.com/orest-d/pointcloud-viewer-rs/blob/main/liquer-pcv/example/server.py).

# Standalone

Pointcloud viewer can as well be run as a standalone desktop application.

PLEASE NOTE: Currently there is a limitation, that the data are always read from the 'data.csv' file.

## Install

If you don't have a rust toolchain, install it as described on the [rust web-site](https://www.rust-lang.org/tools/install):

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then get the source code and build it
```
git clone https://github.com/orest-d/pointcloud-viewer-rs.git
cd pointcloud-viewer-rs
cargo build --release
```

The application can be found in 'target/release' directory.
Copy your data into 'data.csv' in the same directory as the executable before you start it.


