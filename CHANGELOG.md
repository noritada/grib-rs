Version 0.1.0

* Initial release
* Library `grib`
  * Read and basic format checks
  * Decode feature supporting Templates 5.0 and 5.200
* CLI application `gribber` built on the top of the Rust library
  * Display of some information of GRIB2 files

* CLI application `gribber` with following 4 subcommands:
  * decode: data export as text and flat binary files
  * info
  * inspect: display of information mainly for development purpose
  * list
