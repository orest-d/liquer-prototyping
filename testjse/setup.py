import setuptools

long_description = "Test Jupyter Server Extension"

setuptools.setup(
    name="testjse",
    version="0.0.1",
    author="Orest Dubay",
    author_email="orest3.dubay@gmail.com",
    description="Test Jupyter Server Extension",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/orest-d/liquer-prototyping",
    packages=setuptools.find_packages(),
    include_package_data=True,
    zip_safe=False,
    install_requires=[],
    classifiers=[
        "Programming Language :: Python :: 3",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
    ],
)
