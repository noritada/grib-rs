use crate::utils::{cat_to_tempfile, unxz_as_bytes, unxz_to_tempfile};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

fn testdata_dir() -> &'static Path {
    Path::new("testdata")
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

    pub(crate) fn jma_kousa() -> Result<NamedTempFile, io::Error> {
        unxz_to_tempfile(
            testdata_dir()
                .join("Z__C_RJTD_20170221120000_MSG_GPV_Gll0p5deg_Pys_B20170221120000_F2017022115-2017022212_grib2.bin.xz"),
        )
    }

    pub(crate) fn jma_meps() -> Result<NamedTempFile, io::Error> {
        unxz_to_tempfile(
            testdata_dir()
                .join("Z__C_RJTD_20190605000000_MEPS_GPV_Rjp_L-pall_FH00-15_grib2.bin.0-20.xz"),
        )
    }

    pub(crate) fn jma_msmguid() -> Result<NamedTempFile, io::Error> {
        unxz_to_tempfile(
            testdata_dir()
                .join("Z__C_RJTD_20190304000000_MSM_GUID_Rjp_P-all_FH03-39_Toorg_grib2.bin.xz"),
        )
    }

    pub(crate) fn jma_tornado_nowcast() -> Result<NamedTempFile, io::Error> {
        unxz_to_tempfile(
            testdata_dir()
                .join("Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin.xz"),
        )
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
}
