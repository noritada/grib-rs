# Test data

This directory contains test data files.

## Data files from JMA

Following data files are downloaded from [JMA's GPV sample data
page](https://www.data.jma.go.jp/developer/gpv_sample.html), extracted
from zip archives and compressed.

* `Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin.xz`
  (originally in `tornado_170301.zip` for "竜巻発生確度ナウキャスト")
* `Z__C_RJTD_20170221120000_MSG_GPV_Gll0p5deg_Pys_B20170221120000_F2017022115-2017022212_grib2.bin.xz`
  (originally in `kousa_170301.zip` for "黄砂予測モデルGPV")

## Data files generated

Files under the directory `gen` is generated with third-party tools
and compressed.

```
$ wgrib2 -d 1.4 -order we:ns -no_header -ieee tornado-wgrib2-be.bin Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin

$ wgrib2 -d 1.4 -order we:ns -no_header -bin tornado-wgrib2-le.bin Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin

$ wgrib2 -d 1.4 -order we:ns -no_header -ieee kousa-wgrib2-be.bin Z__C_RJTD_20170221120000_MSG_GPV_Gll0p5deg_Pys_B20170221120000_F2017022115-2017022212_grib2.bin

$ wgrib2 -d 1.4 -order we:ns -no_header -bin kousa-wgrib2-le.bin Z__C_RJTD_20170221120000_MSG_GPV_Gll0p5deg_Pys_B20170221120000_F2017022115-2017022212_grib2.bin
```
