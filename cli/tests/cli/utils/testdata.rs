use std::{
    fs::File,
    io::{self, BufReader, Read, Write},
    path::{Path, PathBuf},
};

use tempfile::NamedTempFile;

use crate::utils::{cat_to_tempfile, gzcat_to_tempfile, unxz_as_bytes, xzcat_to_tempfile};

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
        cat_to_tempfile(cmc_glb_file())
    }

    fn cmc_glb_file() -> PathBuf {
        testdata_dir().join("CMC_glb_TMP_ISBL_1_latlon.24x.24_2021051800_P000.grib2")
    }

    fn dwd_icon_file() -> PathBuf {
        testdata_dir().join("icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2")
    }

    pub(crate) fn jma_kousa() -> Result<NamedTempFile, io::Error> {
        xzcat_to_tempfile(
            testdata_dir()
                .join("Z__C_RJTD_20170221120000_MSG_GPV_Gll0p5deg_Pys_B20170221120000_F2017022115-2017022212_grib2.bin.xz"),
        )
    }

    pub(crate) fn jma_meps() -> Result<NamedTempFile, io::Error> {
        xzcat_to_tempfile(
            testdata_dir()
                .join("Z__C_RJTD_20190605000000_MEPS_GPV_Rjp_L-pall_FH00-15_grib2.bin.0-20.xz"),
        )
    }

    pub(crate) fn jma_msmguid() -> Result<NamedTempFile, io::Error> {
        xzcat_to_tempfile(
            testdata_dir()
                .join("Z__C_RJTD_20190304000000_MSM_GUID_Rjp_P-all_FH03-39_Toorg_grib2.bin.xz"),
        )
    }

    pub(crate) fn jma_tornado_nowcast() -> Result<NamedTempFile, io::Error> {
        cat_to_tempfile(
            testdata_dir()
                .join("Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin"),
        )
    }

    pub(crate) fn ncmrwf_wind_solar() -> Result<NamedTempFile, io::Error> {
        xzcat_to_tempfile(testdata_dir().join("wind_solar_ind_0.125_20240521_12Z.grib2.0.xz"))
    }

    pub(crate) fn noaa_gdas_0_10() -> Result<NamedTempFile, io::Error> {
        xzcat_to_tempfile(testdata_dir().join("gdas.t12z.pgrb2.0p25.f000.0-10.xz"))
    }

    pub(crate) fn noaa_gdas_12() -> Result<NamedTempFile, io::Error> {
        xzcat_to_tempfile(testdata_dir().join("gdas.t12z.pgrb2.0p25.f000.12.xz"))
    }

    pub(crate) fn noaa_gdas_46() -> Result<NamedTempFile, io::Error> {
        xzcat_to_tempfile(testdata_dir().join("gdas.t12z.pgrb2.0p25.f000.46.xz"))
    }

    pub(crate) fn noaa_mrms() -> Result<NamedTempFile, io::Error> {
        gzcat_to_tempfile(
            testdata_dir().join("MRMS_ReflectivityAtLowestAltitude_00.50_20230406-120039.grib2.gz"),
        )
    }

    pub(crate) fn noaa_ndfd_critfireo() -> Result<NamedTempFile, io::Error> {
        xzcat_to_tempfile(testdata_dir().join("ds.critfireo.bin.xz"))
    }

    pub(crate) fn noaa_ndfd_minrh() -> Result<NamedTempFile, io::Error> {
        xzcat_to_tempfile(testdata_dir().join("ds.minrh.bin.xz"))
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

    pub(crate) fn cmc_glb_le() -> Result<Vec<u8>, io::Error> {
        unxz_as_bytes(testdata_dir().join("gen").join("cmc-glb-wgrib2-le.bin.xz"))
    }

    pub(crate) fn jma_kousa_be() -> Result<Vec<u8>, io::Error> {
        unxz_as_bytes(testdata_dir().join("gen").join("kousa-wgrib2-be.bin.xz"))
    }

    pub(crate) fn jma_kousa_le() -> Result<Vec<u8>, io::Error> {
        unxz_as_bytes(testdata_dir().join("gen").join("kousa-wgrib2-le.bin.xz"))
    }

    pub(crate) fn jma_meps_le() -> Result<Vec<u8>, io::Error> {
        unxz_as_bytes(testdata_dir().join("gen").join("meps-wgrib2-le.bin.xz"))
    }

    pub(crate) fn jma_msmguid_le() -> Result<Vec<u8>, io::Error> {
        unxz_as_bytes(testdata_dir().join("gen").join("msmguid-wgrib2-le.bin.xz"))
    }

    pub(crate) fn jma_tornado_nowcast_be() -> Result<Vec<u8>, io::Error> {
        unxz_as_bytes(testdata_dir().join("gen").join("tornado-wgrib2-be.bin.xz"))
    }

    pub(crate) fn jma_tornado_nowcast_le() -> Result<Vec<u8>, io::Error> {
        unxz_as_bytes(testdata_dir().join("gen").join("tornado-wgrib2-le.bin.xz"))
    }

    pub(crate) fn ncmrwf_wind_solar_le() -> Result<Vec<u8>, io::Error> {
        unxz_as_bytes(
            testdata_dir()
                .join("gen")
                .join("wind_solar_ind_0.125_20240521_12Z.wgrib2-le.bin.xz"),
        )
    }

    pub(crate) fn noaa_gdas_0_le() -> Result<Vec<u8>, io::Error> {
        unxz_as_bytes(testdata_dir().join("gen").join("gdas-0-wgrib2-le.bin.xz"))
    }

    pub(crate) fn noaa_gdas_1_le() -> Result<Vec<u8>, io::Error> {
        unxz_as_bytes(testdata_dir().join("gen").join("gdas-1-wgrib2-le.bin.xz"))
    }

    pub(crate) fn noaa_gdas_2_le() -> Result<Vec<u8>, io::Error> {
        unxz_as_bytes(testdata_dir().join("gen").join("gdas-2-wgrib2-le.bin.xz"))
    }

    pub(crate) fn noaa_gdas_12_le() -> Result<Vec<u8>, io::Error> {
        unxz_as_bytes(testdata_dir().join("gen").join("gdas-12-wgrib2-le.bin.xz"))
    }

    pub(crate) fn noaa_gdas_46_le() -> Result<Vec<u8>, io::Error> {
        unxz_as_bytes(testdata_dir().join("gen").join("gdas-46-wgrib2-le.bin.xz"))
    }

    pub(crate) fn noaa_mrms_le() -> Result<Vec<u8>, io::Error> {
        unxz_as_bytes(testdata_dir().join("gen").join("mrms-wgrib2-le.bin.xz"))
    }

    pub(crate) fn noaa_ndfd_critfireo_0_le() -> Result<Vec<u8>, io::Error> {
        unxz_as_bytes(testdata_dir().join("gen").join("ds.critfireo.bin.0.xz"))
    }

    pub(crate) fn noaa_ndfd_critfireo_1_le() -> Result<Vec<u8>, io::Error> {
        unxz_as_bytes(testdata_dir().join("gen").join("ds.critfireo.bin.1.xz"))
    }

    pub(crate) fn noaa_ndfd_minrh_0_le() -> Result<Vec<u8>, io::Error> {
        unxz_as_bytes(testdata_dir().join("gen").join("ds.minrh.bin.0.xz"))
    }
}
