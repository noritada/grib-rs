# grib-rs Roadmap

To move this project forward, it is important to clarify the next steps so that everyone knows what is expected in the near future and what is not.

Note that this roadmap lists only larger topics. Smaller topics will be tackled as needed, depending on their priority.

## Now

Currently, as a first step, we are mainly focusing on expanding the basic functionality.

- Submessage selection and data extraction using forecast times, elements, and elevation levels, which requires adequate support for code tables 4.x and tempaltes 4.x
- Data extraction using geographic coordinates, which requires adequate support for code tables 3.x and tempaltes 3.x
- More supports of code tables and templates

## Next

Next step, it is important to do following things:

- API stabilization for the 1.0 release
- Documentation for the 1.0 release

## Future

We are planning to add the following features:

- Support for other versions of GRIB
- Support for GRIB data generation
- WebAssembly application
- Efficient read from cloud sources such as S3
- Format conversion to other popular formats
- Providing interface to other languages
