import setuptools

with open("README.md", "r") as fh:
    long_description = fh.read()

setuptools.setup(
    name="liquer-pcv",
    version="0.3.0",
    author="Orest Dubay",
    author_email="orest3.dubay@gmail.com",
    description="""LiQuer - Pointcloud Viewer is tool for exploratory data analysis.""",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/orest-d/pointcloud-viewer-rs",
    packages=setuptools.find_packages(),
    include_package_data=True,
    zip_safe=False,
    install_requires=['Flask', 'liquer-framework'],
    classifiers=[
        "Programming Language :: Python :: 3",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
    ],
)
