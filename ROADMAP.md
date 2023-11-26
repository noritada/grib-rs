# grib-rs Roadmap

To move this project forward, it is important to clarify the next steps so that everyone knows what is expected in the near future and what is not.

Note that this roadmap lists only larger topics. Smaller topics will be tackled as needed, depending on their priority.

## Current status (as of 0.8.0)

Currently, we are mainly focusing on expanding the basic functionality.

### Reading the overall structure

The current implementation may be changed if any inconvenience arises, but since it works properly for the currently envisioned use, it is considered to be mostly stable.

### Retrieving and showing the attribute values of each submessage (template 4.x)

Retrieving and showing attribute values, such as weather elements, elevation level, forecast time, etc. from each submessage is currently supported, but there is much room for improvement and enhancement, both in terms of functionality and source code.

Since manually adding support would result in a mass of boilerplate code, we are considering creating a proc macro to achieve the functionality with a smaller amount of source code.

### Decoding (templates 5.x and 7.x)

Decoding (unpacking) of 6 packing types are supported. The code is the most well structured and best written in this library crate.

Although support for some packing types and some parameter values is lacking, it is relatively easy to add support if we have data that can be used to validate the decoding.

### Referencing geographic coordinates (templates 3.x)

Currently, only latitude/longitude (or equidistant cylindrical, or plate carree) grid is supported.

A good deal of work needs to be done to add support for conversion from various grid systems to geographic coordinates.

## Next

Next step, it is important to do following things:

- Providing access to all parameters in the data
- Brush up on error messages to clarify missing support
- Core enhancements to facilitate new support for code tables and templates
- API stabilization toward 1.0 release
- Documentation enhancement toward 1.0 release
- Support mechanisms for regionally defined code tables and templates

## Future

We are planning to add the following features:

- Support for other versions of GRIB
- Support for GRIB data generation
- WebAssembly application
- Efficient read from cloud sources such as S3
- Format conversion to other popular formats
- Providing interface to other languages
