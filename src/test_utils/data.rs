macro_rules! definitions_of_test_data {
    (
        $(
            $(#[$meta:meta])*
            ($name:ident, $file_name:expr),
        )*
    ) => ($(
        $(#[$meta])*
        #[allow(dead_code)]
        pub(crate) const $name: &'static str = $file_name;
    )*);
}

pub(crate) mod grib2 {
    definitions_of_test_data! {
        (CMC_GLB, "testdata/CMC_glb_TMP_ISBL_1_latlon.24x.24_2021051800_P000.grib2"),
        /// Notable points: This data includes Section 2 and uses Template 3.101.
        (DWD_ICON, "testdata/icon_global_icosahedral_single-level_2021112018_000_TOT_PREC.grib2"),
        (ECMWF_REALTIME_OPER_FC_0, "testdata/20240101000000-0h-oper-fc.grib2.0-10.xz"),
        (ECMWF_REALTIME_OPER_FC_89, "testdata/20250912120000-0h-oper-fc.grib2.89.xz"),
        (
            JMA_KOUSA,
            "testdata/Z__C_RJTD_20170221120000_MSG_GPV_Gll0p5deg_Pys_B20170221120000_F2017022115-2017022212_grib2.bin.xz"
        ),
        (JMA_MEPS, "testdata/Z__C_RJTD_20190605000000_MEPS_GPV_Rjp_L-pall_FH00-15_grib2.bin.0-20.xz"),
        (JMA_MSMGUID, "testdata/Z__C_RJTD_20190304000000_MSM_GUID_Rjp_P-all_FH03-39_Toorg_grib2.bin.xz"),
        (
            JMA_TORNADO_NOWCAST,
            "testdata/Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin"
        ),
        (NCMRWF_WIND_SOLAR, "testdata/wind_solar_ind_0.125_20240521_12Z.grib2.0.xz"),
        (NOAA_GDAS_0_10, "testdata/gdas.t12z.pgrb2.0p25.f000.0-10.xz"),
        (NOAA_GDAS_12, "testdata/gdas.t12z.pgrb2.0p25.f000.12.xz"),
        (NOAA_GDAS_46, "testdata/gdas.t12z.pgrb2.0p25.f000.46.xz"),
        (NOAA_GDAS_SFLUX, "testdata/gdas.t00z.sfluxgrbf000.grib2.0.xz"),
        (NOAA_MRMS_MERGED_RHO_HV, "testdata/MRMS_MergedRhoHV_19.00_20260219-042039.grib2.gz"),
        (NOAA_MRMS_PRECIP_FLAG, "testdata/MRMS_PrecipFlag_00.00_20260219-042400.grib2.gz"),
        (NOAA_MRMS_REFLECTIVITY, "testdata/MRMS_ReflectivityAtLowestAltitude_00.50_20230406-120039.grib2.gz"),
        (NOAA_NDFD_CRITFIREO, "testdata/ds.critfireo.bin.xz"),
        (NOAA_NDFD_MINRH, "testdata/ds.minrh.bin.xz"),
    }
}

pub(crate) mod flat_binary {
    definitions_of_test_data! {
        (CMC_GLB_LE, "testdata/gen/cmc-glb-wgrib2-le.bin.xz"),
        (ECMWF_REALTIME_OPER_FC_0_LE, "testdata/gen/ecmwf-realtime-oper-fc-0-le.bin.xz"),
        (ECMWF_REALTIME_OPER_FC_89_LE, "testdata/gen/ecmwf-realtime-oper-fc-89-le.bin.xz"),
        (JMA_KOUSA_LE, "testdata/gen/kousa-wgrib2-le.bin.xz"),
        (JMA_MEPS_LE, "testdata/gen/meps-wgrib2-le.bin.xz"),
        (JMA_MSMGUID_LE, "testdata/gen/msmguid-wgrib2-le.bin.xz"),
        (JMA_TORNADO_NOWCAST_LE, "testdata/gen/tornado-wgrib2-le.bin.xz"),
        (NCMRWF_WIND_SOLAR_LE, "testdata/gen/wind_solar_ind_0.125_20240521_12Z.wgrib2-le.bin.xz"),
        (NOAA_GDAS_0_LE, "testdata/gen/gdas-0-wgrib2-le.bin.xz"),
        (NOAA_GDAS_1_LE, "testdata/gen/gdas-1-wgrib2-le.bin.xz"),
        (NOAA_GDAS_2_LE, "testdata/gen/gdas-2-wgrib2-le.bin.xz"),
        (NOAA_GDAS_12_LE, "testdata/gen/gdas-12-wgrib2-le.bin.xz"),
        (NOAA_GDAS_46_LE, "testdata/gen/gdas-46-wgrib2-le.bin.xz"),
        (NOAA_MRMS_MERGED_RHO_HV_LE, "testdata/gen/mrms-merged-rho-hv-wgrib2-le.bin.xz"),
        (NOAA_MRMS_PRECIP_FLAG_LE, "testdata/gen/mrms-precip-flag-wgrib2-le.bin.xz"),
        (NOAA_MRMS_REFLECTIVITY_LE, "testdata/gen/mrms-reflectivity-wgrib2-le.bin.xz"),
        (NOAA_NDFD_CRITFIREO_0_LE, "testdata/gen/ds.critfireo.bin.0.xz"),
        (NOAA_NDFD_CRITFIREO_1_LE, "testdata/gen/ds.critfireo.bin.1.xz"),
        (NOAA_NDFD_MINRH_0_LE, "testdata/gen/ds.minrh.bin.0.xz"),
    }
}
