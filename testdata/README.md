# Test data

This directory contains test data files.

## Data files from CMC

Following data files are downloaded from [CMC's page on meteorological data in
GRIB format](https://weather.gc.ca/grib/index_e.html).
This data is distributed under [Environment and Climate Change Canada Data Server End-use
Licence](https://dd.weather.gc.ca/doc/LICENCE_GENERAL.txt).

* `CMC_glb_TMP_ISBL_1_latlon.24x.24_2021051800_P000.grib2`

## Data files from DWD

Following data files are downloaded from [Open-Data-Server of Deutscher Wetterdienst (DWD)].
The copyright of the data is [held by DWD](https://www.dwd.de/EN/service/copyright/copyright_artikel.html) and distribution is done according to the [rules for acknowledging the DWD as source](https://www.dwd.de/EN/service/copyright/templates_dwd_as_source.html).

```
Source: Deutscher Wetterdienst
```

- `icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2`

## Data files from JMA

Following data files are downloaded from [JMA's GPV sample data
page](https://www.data.jma.go.jp/developer/gpv_sample.html), extracted
from zip archives and compressed.

* `Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin.xz`
  (originally in `tornado_170301.zip` for "竜巻発生確度ナウキャスト")
* `Z__C_RJTD_20170221120000_MSG_GPV_Gll0p5deg_Pys_B20170221120000_F2017022115-2017022212_grib2.bin.xz`
  (originally in `kousa_170301.zip` for "黄砂予測モデルGPV")

Following data file is based on files downloaded from the above page:

* `Z__C_RJTD_20190605000000_MEPS_GPV_Rjp_L-pall_FH00-15_grib2.bin.0-20.xz`
  (Only the first 20 of the 2520 submessages from file
  `Z__C_RJTD_20190605000000_MEPS_GPV_Rjp_L-pall_FH00-15_grib2.bin`,
  originally included in `meps_190627.zip` for "メソアンサンブル予報シ
  ステム（ＭＥＰＳ）ＧＰＶ", were extracted as GRIB2 and compressed.)

## Data files generated

Files under the directory `gen` is generated with third-party tools
and compressed.

```
$ wgrib2 -d 1.4 -order we:ns -no_header -ieee tornado-wgrib2-be.bin Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin

$ wgrib2 -d 1.4 -order we:ns -no_header -bin tornado-wgrib2-le.bin Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin

$ wgrib2 -d 1.4 -order we:ns -no_header -ieee kousa-wgrib2-be.bin Z__C_RJTD_20170221120000_MSG_GPV_Gll0p5deg_Pys_B20170221120000_F2017022115-2017022212_grib2.bin

$ wgrib2 -d 1.4 -order we:ns -no_header -bin kousa-wgrib2-le.bin Z__C_RJTD_20170221120000_MSG_GPV_Gll0p5deg_Pys_B20170221120000_F2017022115-2017022212_grib2.bin

$ wgrib2 -d 1.3 -order we:ns -no_header -bin meps-wgrib2-le.bin Z__C_RJTD_20190605000000_MEPS_GPV_Rjp_L-pall_FH00-15_grib2.bin.0-20

$ wgrib2 -d 1 -no_header -bin cmc-glb-wgrib2-le.bin CMC_glb_TMP_ISBL_1_latlon.24x.24_2021051800_P000.grib2
```
