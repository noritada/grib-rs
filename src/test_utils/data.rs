macro_rules! definitions_of_test_data {
    ($(($name:ident, $file_name:expr),)*) => ($(
        #[allow(dead_code)]
        pub(crate) fn $name() -> &'static str {
            $file_name
        }
    )*);
}

pub(crate) mod grib2 {
    definitions_of_test_data! {
        (cmc_glb, "testdata/CMC_glb_TMP_ISBL_1_latlon.24x.24_2021051800_P000.grib2"),
        (ecmwf_realtime_oper_fc_0, "testdata/20240101000000-0h-oper-fc.grib2.0-10.xz"),
        (ecmwf_realtime_oper_fc_89, "testdata/20250912120000-0h-oper-fc.grib2.89.xz"),
        (
            jma_kousa,
            "testdata/Z__C_RJTD_20170221120000_MSG_GPV_Gll0p5deg_Pys_B20170221120000_F2017022115-2017022212_grib2.bin.xz"
        ),
        (jma_meps, "testdata/Z__C_RJTD_20190605000000_MEPS_GPV_Rjp_L-pall_FH00-15_grib2.bin.0-20.xz"),
        (jma_msmguid, "testdata/Z__C_RJTD_20190304000000_MSM_GUID_Rjp_P-all_FH03-39_Toorg_grib2.bin.xz"),
        (
            jma_tornado_nowcast,
            "testdata/Z__C_RJTD_20160822020000_NOWC_GPV_Ggis10km_Pphw10_FH0000-0100_grib2.bin"
        ),
        (ncmrwf_wind_solar, "testdata/wind_solar_ind_0.125_20240521_12Z.grib2.0.xz"),
        (noaa_gdas_0_10, "testdata/gdas.t12z.pgrb2.0p25.f000.0-10.xz"),
        (noaa_gdas_12, "testdata/gdas.t12z.pgrb2.0p25.f000.12.xz"),
        (noaa_gdas_46, "testdata/gdas.t12z.pgrb2.0p25.f000.46.xz"),
        (noaa_mrms_merged_rho_hv, "testdata/MRMS_MergedRhoHV_19.00_20260219-042039.grib2.gz"),
        (noaa_mrms_precip_flag, "testdata/MRMS_PrecipFlag_00.00_20260219-042400.grib2.gz"),
        (noaa_mrms_reflectivity, "testdata/MRMS_ReflectivityAtLowestAltitude_00.50_20230406-120039.grib2.gz"),
        (noaa_ndfd_critfireo, "testdata/ds.critfireo.bin.xz"),
        (noaa_ndfd_minrh, "testdata/ds.minrh.bin.xz"),
    }
}

pub(crate) mod flat_binary {
    definitions_of_test_data! {
        (cmc_glb_le, "testdata/gen/cmc-glb-wgrib2-le.bin.xz"),
        (ecmwf_realtime_oper_fc_0_le, "testdata/gen/ecmwf-realtime-oper-fc-0-le.bin.xz"),
        (ecmwf_realtime_oper_fc_89_le, "testdata/gen/ecmwf-realtime-oper-fc-89-le.bin.xz"),
        (jma_kousa_le, "testdata/gen/kousa-wgrib2-le.bin.xz"),
        (jma_meps_le, "testdata/gen/meps-wgrib2-le.bin.xz"),
        (jma_msmguid_le, "testdata/gen/msmguid-wgrib2-le.bin.xz"),
        (jma_tornado_nowcast_le, "testdata/gen/tornado-wgrib2-le.bin.xz"),
        (ncmrwf_wind_solar_le, "testdata/gen/wind_solar_ind_0.125_20240521_12Z.wgrib2-le.bin.xz"),
        (noaa_gdas_0_le, "testdata/gen/gdas-0-wgrib2-le.bin.xz"),
        (noaa_gdas_1_le, "testdata/gen/gdas-1-wgrib2-le.bin.xz"),
        (noaa_gdas_2_le, "testdata/gen/gdas-2-wgrib2-le.bin.xz"),
        (noaa_gdas_12_le, "testdata/gen/gdas-12-wgrib2-le.bin.xz"),
        (noaa_gdas_46_le, "testdata/gen/gdas-46-wgrib2-le.bin.xz"),
        (noaa_mrms_merged_rho_hv_le, "testdata/gen/mrms-merged-rho-hv-wgrib2-le.bin.xz"),
        (noaa_mrms_precip_flag_le, "testdata/gen/mrms-precip-flag-wgrib2-le.bin.xz"),
        (noaa_mrms_reflectivity_le, "testdata/gen/mrms-reflectivity-wgrib2-le.bin.xz"),
        (noaa_ndfd_critfireo_0_le, "testdata/gen/ds.critfireo.bin.0.xz"),
        (noaa_ndfd_critfireo_1_le, "testdata/gen/ds.critfireo.bin.1.xz"),
        (noaa_ndfd_minrh_0_le, "testdata/gen/ds.minrh.bin.0.xz"),
    }
}
