use std::{
    fs::File,
    io::{self, BufReader, Read, Write},
    path::{Path, PathBuf},
};

use tempfile::NamedTempFile;

use crate::utils::{get_uncompressed, write_uncompressed_to_tempfile};

fn testdata_dir() -> PathBuf {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/..")).join("testdata")
}

#[inline]
pub(crate) fn empty_file() -> Result<NamedTempFile, io::Error> {
    NamedTempFile::new()
}

pub(crate) fn too_small_file() -> Result<NamedTempFile, io::Error> {
    let mut out = NamedTempFile::new()?;
    out.write_all(b"foo")?;

    Ok(out)
}

pub(crate) fn non_grib_file() -> Result<NamedTempFile, io::Error> {
    let mut out = NamedTempFile::new()?;
    out.write_all(b"foo foo foo foo foo foo foo foo ")?;

    Ok(out)
}

pub(crate) mod grib2 {
    use super::*;

    pub(crate) fn cmc_glb() -> Result<NamedTempFile, io::Error> {
        write_uncompressed_to_tempfile(cmc_glb_file())
    }

    fn cmc_glb_file() -> PathBuf {
        testdata_dir().join("CMC_glb_TMP_ISBL_1_latlon.24x.24_2021051800_P000.grib2")
    }

    fn dwd_icon_file() -> PathBuf {
        testdata_dir().join("icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2")
    }

    macro_rules! definitions_of_grib2_test_data {
        ($(($name:ident, $file_name:expr),)*) => ($(
            pub(crate) fn $name() -> Result<NamedTempFile, io::Error> {
                write_uncompressed_to_tempfile(testdata_dir().join($file_name))
            }
        )*);
    }

    definitions_of_grib2_test_data! {
        (
            jma_kousa,
            "Z__C_RJTD_20170221120000_MSG_GPV_Gll0p5deg_Pys_B20170221120000_F2017022115-2017022212_grib2.bin.xz"
        ),
        (jma_meps, "Z__C_RJTD_20190605000000_MEPS_GPV_Rjp_L-pall_FH00-15_grib2.bin.0-20.xz"),
        (jma_msmguid, "Z__C_RJTD_20190304000000_MSM_GUID_Rjp_P-all_FH03-39_Toorg_grib2.bin.xz"),
        (
            jma_tornado_nowcast,
            "Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin"
        ),
        (ncmrwf_wind_solar, "wind_solar_ind_0.125_20240521_12Z.grib2.0.xz"),
        (noaa_gdas_0_10, "gdas.t12z.pgrb2.0p25.f000.0-10.xz"),
        (noaa_gdas_12, "gdas.t12z.pgrb2.0p25.f000.12.xz"),
        (noaa_gdas_46, "gdas.t12z.pgrb2.0p25.f000.46.xz"),
        (noaa_mrms, "MRMS_ReflectivityAtLowestAltitude_00.50_20230406-120039.grib2.gz"),
        (noaa_ndfd_critfireo, "ds.critfireo.bin.xz"),
        (noaa_ndfd_minrh, "ds.minrh.bin.xz"),
    }

    pub(crate) fn multi_message_data(n: usize) -> Result<NamedTempFile, io::Error> {
        let mut buf = Vec::new();
        let mut out = NamedTempFile::new()?;

        let f = File::open(dwd_icon_file())?;
        let mut f = BufReader::new(f);
        f.read_to_end(&mut buf)?;
        for _ in 0..n {
            out.write_all(&buf)?;
        }

        Ok(out)
    }
}

pub(crate) mod flat_binary {
    use super::*;

    macro_rules! definitions_of_flat_binary_test_data {
        ($(($name:ident, $file_name:expr),)*) => ($(
            pub(crate) fn $name() -> Result<Vec<u8>, io::Error> {
                get_uncompressed(testdata_dir().join("gen").join($file_name))
            }
        )*);
    }

    definitions_of_flat_binary_test_data! {
        (cmc_glb_le, "cmc-glb-wgrib2-le.bin.xz"),
        (jma_kousa_be, "kousa-wgrib2-be.bin.xz"),
        (jma_kousa_le, "kousa-wgrib2-le.bin.xz"),
        (jma_meps_le, "meps-wgrib2-le.bin.xz"),
        (jma_msmguid_le, "msmguid-wgrib2-le.bin.xz"),
        (jma_tornado_nowcast_be, "tornado-wgrib2-be.bin.xz"),
        (jma_tornado_nowcast_le, "tornado-wgrib2-le.bin.xz"),
        (ncmrwf_wind_solar_le, "wind_solar_ind_0.125_20240521_12Z.wgrib2-le.bin.xz"),
        (noaa_gdas_0_le, "gdas-0-wgrib2-le.bin.xz"),
        (noaa_gdas_1_le, "gdas-1-wgrib2-le.bin.xz"),
        (noaa_gdas_2_le, "gdas-2-wgrib2-le.bin.xz"),
        (noaa_gdas_12_le, "gdas-12-wgrib2-le.bin.xz"),
        (noaa_gdas_46_le, "gdas-46-wgrib2-le.bin.xz"),
        (noaa_mrms_le, "mrms-wgrib2-le.bin.xz"),
        (noaa_ndfd_critfireo_0_le, "ds.critfireo.bin.0.xz"),
        (noaa_ndfd_critfireo_1_le, "ds.critfireo.bin.1.xz"),
        (noaa_ndfd_minrh_0_le, "ds.minrh.bin.0.xz"),
    }
}
