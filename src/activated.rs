use std::cell::LazyCell;
use std::ptr::null_mut;

use anyhow::bail;
use windows::Win32::Security::Authentication::Identity::{
    SL_GEN_STATE_INVALID_LICENSE, SL_GEN_STATE_IS_GENUINE, SLClose, SLGetSLIDList, SLIsGenuineLocal,
};
use windows::Win32::Security::Authentication::Identity::{
    SL_ID_APPLICATION, SL_ID_PRODUCT_SKU, SLOpen,
};
use windows_core::GUID;

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Default)]
pub enum ActivationType {
    #[default]
    Unlicensed,
    LikelyGenuine,
    Pirated,
}

// 55c92734-d682-4d71-983e-d6ec3f16059f
const WIN_APP_GUID: GUID = GUID::from_values(
    0x55c92734,
    0xd682,
    0x4d71,
    [0x98, 0x3e, 0xd6, 0xec, 0x3f, 0x16, 0x05, 0x9f],
);

#[allow(clippy::similar_names)]
pub fn get_windows_license_type() -> anyhow::Result<ActivationType> {
    let mut hslc = HslcManager::new();
    unsafe {
        hslc.open()?;
    }

    if hslc.0.is_null() {
        bail!("hslc null");
    }

    let mut pn_return_ids = 0u32;
    let mut pp_return_ids: *mut GUID = null_mut();

    unsafe {
        SLGetSLIDList(
            hslc.0,
            SL_ID_APPLICATION,
            Some(&WIN_APP_GUID),
            SL_ID_PRODUCT_SKU,
            &raw mut pn_return_ids,
            &raw mut pp_return_ids,
        )?;
    }

    if pp_return_ids.is_null() {
        bail!("ppris null");
    }

    for i in 0..pn_return_ids {
        let guid = unsafe { *pp_return_ids.add(i as usize) };
        if KEYS.with(|keys| keys.contains(&guid)) {
            return Ok(ActivationType::Pirated);
        }
    }

    if check_is_activated().unwrap_or_default() {
        Ok(ActivationType::LikelyGenuine)
    } else {
        Ok(ActivationType::Unlicensed)
    }
}

pub fn check_is_activated() -> anyhow::Result<bool> {
    let mut sl_genuine_state = SL_GEN_STATE_INVALID_LICENSE;

    unsafe {
        SLIsGenuineLocal(&WIN_APP_GUID, &raw mut sl_genuine_state, None)?;
    }

    Ok(sl_genuine_state == SL_GEN_STATE_IS_GENUINE)
}

struct HslcManager(pub *mut core::ffi::c_void);

impl HslcManager {
    pub const fn new() -> Self {
        Self(null_mut())
    }

    pub unsafe fn open(&mut self) -> windows_core::Result<()> {
        unsafe { SLOpen(&raw mut self.0) }
    }
}

impl Drop for HslcManager {
    fn drop(&mut self) {
        let _ = unsafe { SLClose(self.0) };
    }
}

// https://github.com/massgravel/Microsoft-Activation-Scripts/blob/5ad6226f37e7af75807819d669ff5ae0e3421a87/MAS/Separate-Files-Version/Activators/HWID_Activation.cmd#L1751
// https://github.com/massgravel/Microsoft-Activation-Scripts/blob/5ad6226f37e7af75807819d669ff5ae0e3421a87/MAS/Separate-Files-Version/Activators/HWID_Activation.cmd#L1829

// https://github.com/massgravel/Microsoft-Activation-Scripts/blob/5ad6226f37e7af75807819d669ff5ae0e3421a87/MAS/Separate-Files-Version/Activators/TSforge_Activation.cmd#L986
// https://github.com/massgravel/Microsoft-Activation-Scripts/blob/5ad6226f37e7af75807819d669ff5ae0e3421a87/MAS/Separate-Files-Version/Activators/TSforge_Activation.cmd#L1099

// https://github.com/massgravel/Microsoft-Activation-Scripts/blob/5ad6226f37e7af75807819d669ff5ae0e3421a87/MAS/Separate-Files-Version/Activators/Online_KMS_Activation.cmd#L3663
// https://github.com/massgravel/Microsoft-Activation-Scripts/blob/5ad6226f37e7af75807819d669ff5ae0e3421a87/MAS/Separate-Files-Version/Activators/Online_KMS_Activation.cmd#L3962

// https://github.com/massgravel/Microsoft-Activation-Scripts/blob/5ad6226f37e7af75807819d669ff5ae0e3421a87/MAS/Separate-Files-Version/Activators/KMS38_Activation.cmd#L1890
// https://github.com/massgravel/Microsoft-Activation-Scripts/blob/5ad6226f37e7af75807819d669ff5ae0e3421a87/MAS/Separate-Files-Version/Activators/KMS38_Activation.cmd#L1955

const RAW_KEYS: &[&str] = &[
    "8b351c9c-f398-4515-9900-09df49427262_XGVPP-NMH47-7TTHJ-W3FW7-8H%f%V2C___4_X19-99683_HGNKjkKcKQHO6n8srMUrDh/MElffBZarLqCMD9rWtgFKf3YzYOLDPEMGhuO/auNMKCeiU7ebFbQALS/MyZ7TvidMQ2dvzXeXXKzPBjfwQx549WJUU7qAQ9Txg9cR9SAT8b12Pry2iBk+nZWD9VtHK3kOnEYkvp5WTCTsrSi6Re4_0_OEM:NONSLP_Enterprise",
    "c83cef07-6b72-4bbc-a28f-a00386872839_3V6Q6-NQXCX-V8YXR-9QCYV-QP%f%FCT__27_X19-98746_NHn2n0N1UfVf00CfaI5LCDMDsKdVAWpD/HAfUrcTAKsw9d2Sks4h5MhyH/WUx+B6dFi8ol7D3AHorR8y9dqVS1Bd2FdZNJl/tTR1PGwYn6KL88NS19aHmFNdX8s4438vaa+Ty8Qk8EDcwm/wscC8lQmi3/RgUKYdyGFvpbGSVlk_0_Volume:MAK_EnterpriseN",
    "4de7cb65-cdf1-4de9-8ae8-e3cce27b9f2c_VK7JG-NPHTM-C97JM-9MPGT-3V%f%66T__48_X19-98841_Yl/jNfxJ1SnaIZCIZ4m6Pf3ySNoQXifNeqfltNaNctx+onwiivOx7qcSn8dFtURzgMzSOFnsRQzb5IrvuqHoxWWl1S3JIQn56FvKsvSx7aFXIX3+2Q98G1amPV/WEQ0uHA5d7Ya6An+g0Z0zRP7evGoomTs4YuweaWiZQjQzSpA_0_____Retail_Professional",
    "9fbaf5d6-4d83-4422-870d-fdda6e5858aa_2B87N-8KFHP-DKV6R-Y2C8J-PK%f%CKT__49_X19-98859_Ge0mRQbW8ALk7T09V+1k1yg66qoS0lhkgPIROOIOgxKmWPAvsiLAYPKDqM4+neFCA/qf1dHFmdh0VUrwFBPYsK251UeWuElj4bZFVISL6gUt1eZwbGfv5eurQ0i+qZiFv+CcQOEFsd5DD4Up6xPLLQS3nAXODL5rSrn2sHRoCVY_0_____Retail_ProfessionalN",
    "f742e4ff-909d-4fe9-aacb-3231d24a0c58_4CPRK-NM3K3-X6XXQ-RXX86-WX%f%CHW__98_X19-98877_vel4ytVtnE8FhvN87Cflz9sbh5QwHD1YGOeej9QP7hF3vlBR4EX2/S/09gRneeXVbQnjDOCd2KFMKRUWHLM7ZhFBk8AtlG+kvUawPZ+CIrwrD3mhi7NMv8UX/xkLK3HnBupMEuEwsMJgCUD8Pn6om1mEiQebHBAqu4cT7GN9Y0g_0_____Retail_CoreN",
    "1d1bac85-7365-4fea-949a-96978ec91ae0_N2434-X9D7W-8PF6X-8DV9T-8T%f%YMD__99_X19-99652_Nv17eUTrr1TmUX6frlI7V69VR6yWb7alppCFJPcdjfI+xX4/Cf2np3zm7jmC+zxFb9nELUs477/ydw2KCCXFfM53bKpBQZKHE5+MdGJGxebOCcOtJ3hrkDJtwlVxTQmUgk5xnlmpk8PHg82M2uM5B7UsGLxGKK4d3hi0voSyKeI_0_____Retail_CoreCountrySpecific",
    "3ae2cc14-ab2d-41f4-972f-5e20142771dc_BT79Q-G7N6G-PGBYW-4YWX6-6F%f%4BT_100_X19-99661_FV2Eao/R5v8sGrfQeOjQ4daokVlNOlqRCDZXuaC45bQd5PsNU3t1b4AwWeYM8TAwbHauzr4tPG0UlsUqUikCZHy0poROx35bBBMBym6Zbm9wDBVyi7nCzBtwS86eOonQ3cU6WfZxhZRze0POdR33G3QTNPrnVIM2gf6nZJYqDOA_0_____Retail_CoreSingleLanguage",
    "2b1f36bb-c1cd-4306-bf5c-a0367c2d97d8_YTMG3-N6DKC-DKB77-7M9GH-8H%f%VX7_101_X19-98868_GH/jwFxIcdQhNxJIlFka8c1H48PF0y7TgJwaryAUzqSKXynONLw7MVciDJFVXTkCjbXSdxLSWpPIC50/xyy1rAf8aC7WuN/9cRNAvtFPC1IVAJaMeq1vf4mCqRrrxJQP6ZEcuAeHFzLe/LLovGWCd8rrs6BbBwJXCvAqXImvycQ_0_____Retail_Core",
    "2a6137f3-75c0-4f26-8e3e-d83d802865a4_XKCNC-J26Q9-KFHD2-FKTHY-KD%f%72Y_119_X19-99606_hci78IRWDLBtdbnAIKLDgV9whYgtHc1uYyp9y6FszE9wZBD5Nc8CUD2pI2s2RRd3M04C4O7M3tisB3Ov/XVjpAbxlX3MWfUR5w4MH0AphbuQX0p5MuHEDYyfqlRgBBRzOKePF06qfYvPQMuEfDpKCKFwNojQxBV8O0Arf5zmrIw_0_OEM:NONSLP_PPIPro",
    "e558417a-5123-4f6f-91e7-385c1c7ca9d4_YNMGQ-8RYV3-4PGQ3-C8XTP-7C%f%FBY_121_X19-98886_x9tPFDZmjZMf29zFeHV5SHbXj8Wd8YAcCn/0hbpLcId4D7OWqkQKXxXHIegRlwcWjtII0sZ6WYB0HQV2KH3LvYRnWKpJ5SxeOgdzBIJ6fhegYGGyiXsBv9sEb3/zidPU6ZK9LugVGAcRZ6HQOiXyOw+Yf5H35iM+2oDZXSpjvJw_0_____Retail_Education",
    "c5198a66-e435-4432-89cf-ec777c9d0352_84NGF-MHBT6-FXBX8-QWJK7-DR%f%R8H_122_X19-98892_jkL4YZkmBCJtvL1fT30ZPBcjmzshBSxjwrE0Q00AZ1hYnhrH+npzo1MPCT6ZRHw19ZLTz7wzyBb0qqcBVbtEjZW0Xs2MYLxgriyoONkhnPE6KSUJBw7C0enFVLHEqnVu/nkaOFfockN3bc+Eouw6W2lmHjklPHc9c6Clo04jul0_0_____Retail_EducationN",
    "f6e29426-a256-4316-88bf-cc5b0f95ec0c_PJB47-8PN2T-MCGDY-JTY3D-CB%f%CPV_125_X23-50331_OPGhsyx+Ctw7w/KLMRNrY+fNBmKPjUG0R9RqkWk4e8ez+ExSJxSLLex5WhO5QSNgXLmEra+cCsN6C638aLjIdH2/L7D+8z/C6EDgRvbHMmidHg1lX3/O8lv0JudHkGtHJYewjorn/xXGY++vOCTQdZNk6qzEgmYSvPehKfdg8js_1_Volume:MAK_EnterpriseS_Ge",
    "cce9d2de-98ee-4ce2-8113-222620c64a27_KCNVH-YKWX8-GJJB9-H9FDT-6F%f%7W2_125_X22-66075_GCqWmJOsTVun9z4QkE9n2XqBvt3ZWSPl9QmIh9Q2mXMG/QVt2IE7S+ES/NWlyTSNjLVySr1D2sGjxgEzy9kLwn7VENQVJ736h1iOdMj/3rdqLMSpTa813+nPSQgKpqJ3uMuvIvRP0FdB7Y4qt8qf9kNKK25A1QknioD/6YubL/4_1_Volume:MAK_EnterpriseS_VB",
    "d06934ee-5448-4fd1-964a-cd077618aa06_43TBQ-NH92J-XKTM7-KT3KK-P3%f%9PB_125_X21-83233_EpB6qOCo8pRgO5kL4vxEHck2J1vxyd9OqvxUenDnYO9AkcGWat/D74ZcFg5SFlIya1U8l5zv+tsvZ4wAvQ1IaFW1PwOKJLOaGgejqZ41TIMdFGGw+G+s1RHsEnrWr3UOakTodby1aIMUMoqf3NdaM5aWFo8fOmqWC5/LnCoighs_0_OEM:NONSLP_EnterpriseS_RS5",
    "706e0cfd-23f4-43bb-a9af-1a492b9f1302_NK96Y-D9CD8-W44CQ-R8YTK-DY%f%JWX_125_X21-05035_ntcKmazIvLpZOryft28gWBHu1nHSbR+Gp143f/BiVe+BD2UjHBZfSR1q405xmQZsygz6VRK6+zm8FPR++71pkmArgCLhodCQJ5I4m7rAJNw/YX99pILphi1yCRcvHsOTGa825GUVXgf530tHT6hr0HQ1lGeGgG1hPekpqqBbTlg_0_OEM:NONSLP_EnterpriseS_RS1",
    "faa57748-75c8-40a2-b851-71ce92aa8b45_FWN7H-PF93Q-4GGP8-M8RF3-MD%f%WWW_125_X19-99617_Fe9CDClilrAmwwT7Yhfx67GafWRQEpwyj8R+a4eaTqbpPcAt7d1hv1rx8Sa9AzopEGxIrb7IhiPoDZs0XaT1HN0/olJJ/MnD73CfBP4sdQdLTsSJE3dKMWYTQHpnjqRaS/pNBYRr8l9Mv8yfcP8uS2MjIQ1cRTqRmC7WMpShyCg_0_OEM:NONSLP_EnterpriseS_TH",
    "3d1022d8-969f-4222-b54b-327f5a5af4c9_2DBW3-N2PJG-MVHW3-G7TDK-9H%f%KR4_126_X21-04921_zLPNvcl1iqOefy0VLg+WZgNtRNhuGpn8+BFKjMqjaNOSKiuDcR6GNDS5FF1Aqk6/e6shJ+ohKzuwrnmYq3iNQ3I2MBlYjM5kuNfKs8Vl9dCjSpQr//GBGps6HtF2xrG/2g/yhtYC7FbtGDIE16uOeNKFcVg+XMb0qHE/5Etyfd8_0_Volume:MAK_EnterpriseSN_RS1",
    "60c243e1-f90b-4a1b-ba89-387294948fb6_NTX6B-BRYC2-K6786-F6MVQ-M7%f%V2X_126_X19-98770_kbXfe0z9Vi1S0yfxMWzI5+UtWsJKzxs7wLGUDLjrckFDn1bDQb4MvvuCK1w+Qrq33lemiGpNDspa+ehXiYEeSPFcCvUBpoMlGBFfzurNCHWiv3o1k3jBoawJr/VoDoVZfxhkps0fVoubf9oy6C6AgrkZ7PjCaS58edMcaUWvYYg_0_Volume:MAK_EnterpriseSN_TH",
    "01eb852c-424d-4060-94b8-c10d799d7364_3XP6D-CRND4-DRYM2-GM84D-4G%f%G8Y_139_X23-37869_PVW0XnRJnsWYjTqxb6StCi2tge/uUwegjdiFaFUiZpwdJ620RK+MIAsSq5S+egXXzIWNntoy2fB6BO8F1wBFmxP/mm/3rn5C33jtF5QrbNqY7X9HMbqSiC7zhs4v4u2Xa4oZQx8JQkwr8Q2c/NgHrOJKKRASsSckhunxZ+WVEuM_1_____Retail_ProfessionalCountrySpecific_Zn",
    "eb6d346f-1c60-4643-b960-40ec31596c45_DXG7C-N36C4-C4HTG-X4T3X-2Y%f%V77_161_X21-43626_MaVqTkRrGnOqYizl15whCOKWzx01+BZTVAalvEuHXM+WV55jnIfhWmd/u1GqCd5OplqXdU959zmipK2Iwgu2nw/g91nW//sQiN/cUcvg1Lxo6pC3gAo1AjTpHmGIIf9XlZMYlD+Vl6gXsi/Auwh3yrSSFh5s7gOczZoDTqQwHXA_0_____Retail_ProfessionalWorkstation",
    "89e87510-ba92-45f6-8329-3afa905e3e83_WYPNQ-8C467-V2W6J-TX4WX-WT%f%2RQ_162_X21-43644_JVGQowLiCcPtGY9ndbBDV+rTu/q5ljmQTwQWZgBIQsrAeQjLD8jLEk/qse7riZ7tMT6PKFVNXeWqF7PhLAmACbE8O3Lvp65XMd/Oml9Daynj5/4n7unsffFHIHH8TGyO5j7xb4dkFNqC5TX3P8/1gQEkTIdZEOTQQXFu0L2SP5c_0_____Retail_ProfessionalWorkstationN",
    "62f0c100-9c53-4e02-b886-a3528ddfe7f6_8PTT6-RNW4C-6V7J2-C2D3X-MH%f%BPB_164_X21-04955_CEDgxI8f/fxMBiwmeXw5Of55DG32sbGALzHihXkdbYTDaE3pY37oAA4zwGHALzAFN/t254QImGPYR6hATgl+Cp804f7serJqiLeXY965Zy67I4CKIMBm49lzHLFJeDnVTjDB0wVyN29pvgO3+HLhZ22KYCpkRHFFMy2OKxS68Yc_0_____Retail_ProfessionalEducation",
    "13a38698-4a49-4b9e-8e83-98fe51110953_GJTYN-HDMQY-FRR76-HVGC7-QP%f%F8P_165_X21-04956_r35zp9OfxKSBcTxKWon3zFtbOiCufAPo6xRGY5DJqCRFKdB0jgZalNQitvjmaZ/Rlez2vjRJnEart4LrvyW4d9rrukAjR3+c3UkeTKwoD3qBl9AdRJbXCa2BdsoXJs1WVS4w4LuVzpB/SZDuggZt0F2DlMB427F5aflook/n1pY_0_____Retail_ProfessionalEducationN",
    "df96023b-dcd9-4be2-afa0-c6c871159ebe_NJCF7-PW8QT-3324D-688JX-2Y%f%V66_175_X21-41295_rVpetYUmiRB48YJfCvJHiaZapJ0bO8gQDRoql+rq5IobiSRu//efV1VXqVpBkwILQRKgKIVONSTUF5y2TSxlDLbDSPKp7UHfbz17g6vRKLwOameYEz0ZcK3NTbApN/cMljHvvF/mBag1+sHjWu+eoFzk8H89k9nw8LMeVOPJRDc_0_____Retail_ServerRdsh",
    "d4ef7282-3d2c-4cf0-9976-8854e64a8d1e_V3WVW-N2PV2-CGWC3-34QGF-VM%f%J2C_178_X21-32983_Xzme9hDZR6H0Yx0deURVdE6LiTOkVqWng5W/OTbkxRc0rq+mSYpo/f/yqhtwYlrkBPWx16Yok5Bvcb34vbKHvEAtxfYp4te20uexLzVOtBcoeEozARv4W/6MhYfl+llZtR5efsktj4N4/G4sVbuGvZ9nzNfQO9TwV6NGgGEj2Ec_0_____Retail_Cloud",
    "af5c9381-9240-417d-8d35-eb40cd03e484_NH9J3-68WK7-6FB93-4K3DF-DJ%f%4F6_179_X21-32987_QGRDZOU/VZhYLOSdp2xDnFs8HInNZctcQlWCIrORVnxTQr55IJwN4vK3PJHjkfRLQ/bgUrcEIhyFbANqZFUq8yD1YNubb2bjNORgI/m8u85O9V7nDGtxzO/viEBSWyEHnrzLKKWYqkRQKbbSW3ungaZR0Ti5O2mAUI4HzAFej50_0_____Retail_CloudN",
    "8ab9bdd1-1f67-4997-82d9-8878520837d9_XQQYW-NFFMW-XJPBH-K8732-CK%f%FFD_188_X21-99378_djy0od0uuKd2rrIl+V1/2+MeRltNgW7FEeTNQsPMkVSL75NBphgoso4uS0JPv2D7Y1iEEvmVq6G842Kyt52QOwXgFWmP/IQ6Sq1dr+fHK/4Et7bEPrrGBEZoCfWqk0kdcZRPBij2KN6qCRWhrk1hX2g+U40smx/EYCLGh9HCi24_0_____OEM:DM_IoTEnterprise",
    "ed655016-a9e8-4434-95d9-4345352c2552_QPM6N-7J2WJ-P88HH-P3YRH-YY%f%74H_191_X21-99682_qHs/PzfhYWdtSys2edzcz4h+Qs8aDqb8BIiQ/mJ/+0uyoJh1fitbRCIgiFh2WAGZXjdgB8hZeheNwHibd8ChXaXg4u+0XlOdFlaDTgTXblji8fjETzDBk9aGkeMCvyVXRuUYhTSdp83IqGHz7XuLwN2p/6AUArx9JZCoLGV8j3w_0_OEM:NONSLP_IoTEnterpriseS_VB",
    "6c4de1b8-24bb-4c17-9a77-7b939414c298_CGK42-GYN6Y-VD22B-BX98W-J8%f%JXD_191_X23-12617_J/fpIRynsVQXbp4qZNKp6RvOgZ/P2klILUKQguMlcwrBZybwNkHg/kM5LNOF/aDzEktbPnLnX40GEvKkYT6/qP4cMhn/SOY0/hYOkIdR34ilzNlVNq5xP7CMjCjaUYJe+6ydHPK6FpOuEoWOYYP5BZENKNGyBy4w4shkMAw19mA_0_OEM:NONSLP_IoTEnterpriseS_Ge",
    "d4bdc678-0a4b-4a32-a5b3-aaa24c3b0f24_K9VKN-3BGWV-Y624W-MCRMQ-BH%f%DCD_202_X22-53884_kyoNx2s93U6OUSklB1xn+GXcwCJO1QTEtACYnChi8aXSoxGQ6H2xHfUdHVCwUA1OR0UeNcRrMmOzZBOEUBtdoGWSYPg9AMjvxlxq9JOzYAH+G6lT0UbCWgMSGGrqdcIfmshyEak3aUmsZK6l+uIAFCCZZ/HbbCRkkHC5rWKstMI_0_____Retail_CloudEditionN",
    "92fb8726-92a8-4ffc-94ce-f82e07444653_KY7PN-VR6RX-83W6Y-6DDYQ-T6%f%R4W_203_X22-53847_gD6HnT4jP4rcNu9u83gvDiQq1xs7QSujcDbo60Di5iSVa9/ihZ7nlhnA0eDEZfnoDXriRiPPqc09T6AhSnFxLYitAkOuPJqL5UMobIrab9dwTKlowqFolxoHhLOO4V92Hsvn/9JLy7rEzoiAWHhX/0cpMr3FCzVYPeUW1OyLT1A_0_____Retail_CloudEdition",
    "5a85300a-bfce-474f-ac07-a30983e3fb90_N979K-XWD77-YW3GB-HBGH6-D3%f%2MH_205_X23-15042_blZopkUuayCTgZKH4bOFiisH9GTAHG5/js6UX/qcMWWc3sWNxKSX1OLp1k3h8Xx1cFuvfG/fNAw/I83ssEtPY+A0Gx1JF4QpRqsGOqJ5ruQ2tGW56CJcCVHkB+i46nJAD759gYmy3pEYMQbmpWbhLx3MJ6kvwxKfU+0VCio8k50_0_____OEM:DM_IoTEnterpriseSK",
    "80083eae-7031-4394-9e88-4901973d56fe_P8Q7T-WNK7X-PMFXY-VXHBG-RR%f%K69_206_X23-62084_habUJ0hhAG0P8iIKaRQ74/wZQHyAdFlwHmrejNjOSRG08JeqilJlTM6V8G9UERLJ92/uMDVHIVOPXfN8Zdh8JuYO8oflPnqymIRmff/pU+Gpb871jV2JDA4Cft5gmn+ictKoN4VoSfEZRR+R5hzF2FsoCExDNNw6gLdjtiX94uA_0_____OEM:DM_IoTEnterpriseK",
    "d9eea459-1e6b-499d-8486-e68163f2a8be_N3QJR-YCWKK-RVJGK-GQFMX-T8%f%2BF_EmbeddedIndustryEval_8.1",
    "fbd4c5c6-adc6-4740-bc65-b2dc6dc249c1_MJ8TN-42JH8-886MT-8THCF-36%f%67B_EnterpriseEval_8_NoAct_ ",
    "0eebbb45-29d4-49cb-ba87-a23db0cce40a_76FKW-8NR3K-QDH4P-3C87F-JH%f%TTW_EnterpriseEval_8.1",
    "3f4c0546-36c6-46a8-a37f-be13cdd0cf25_7HBDQ-QNKVG-K4RBF-HMBY6-YG%f%9R6_EnterpriseEval_10",
    "1f8dbfe8-defa-4676-b5a6-f76949a01540_4N8VT-7Y686-43DGV-THTW9-M9%f%8W7_EnterpriseNEval_10",
    "57a4ebb6-8e0c-41f8-b79e-8872ddc971ef_W63GF-7N4D9-GQH3K-K4FP7-9B%f%T6C_EnterpriseSEval_2015",
    "b47dd250-fd6a-44c8-9217-03aca6e4812e_N4DMT-RJKDQ-XR6H7-3DKKP-3Y%f%JWT_EnterpriseSEval_2016",
    "267bf82d-08e8-4046-b061-9ef3f8ac2b5a_N7HMH-MK36Q-M4X93-76KQ2-6J%f%HWR_EnterpriseSEval_2019",
    "aff25f1f-fb53-4e27-95ef-b8e5aca10ac6_9V4NK-624Y3-VK47R-Q27GP-27%f%PGF_EnterpriseSEval_2021",
    "399f0697-886b-4881-894c-4ff6c52e7d8f_CYPB3-XNV9V-QR4G4-Q3B8K-KQ%f%FGJ_EnterpriseSEval_2024",
    "6162e8c2-3c30-46e1-b964-0de603498e2d_R34N9-HJ6Q3-GBX4F-Q24KQ-49%f%DF7_EnterpriseSNEval_2016",
    "aed14fc8-907d-44fb-a3a1-d5d8e638acb3_MHN9Q-RD9PW-BFHDQ-9FTWQ-WQ%f%PF8_EnterpriseSNEval_2019",
    "5dd0c869-eae9-40ce-af48-736692cd8e43_XCN62-29X92-C4T8X-WP82X-DY%f%MJ8_EnterpriseSNEval_2021",
    "522cc0dc-3c7b-4258-ae68-f297ca63b64e_Y8DJM-NPXF3-QG4MH-W7WJK-KQ%f%FGM_EnterpriseSNEval_2024",
    "aa708397-8618-42de-b120-a44190ef456d_R63DV-9NPDX-QVWJF-HMR8V-M4%f%K7D_IoTEnterpriseSEval_2024",
    "cd25b1e8-5839-4a96-a769-b6abe3aa5dee_73BMN-332G9-DX6B8-FGDT3-GF%f%YT6_ServerDatacenterEval_2012",
    "e628c5e8-2300-4429-8b80-a8b21bd7ce0a_WPR94-KN3J7-MRB7X-JPJV8-RX%f%7J2_ServerDatacenterEval_2012R2",
    "01398239-85ff-487f-9e90-0e3cc5bcc92e_QVTQ9-GNRBH-JQ9G7-W7FBW-RX%f%9QR_ServerDatacenterEval_2016",
    "5ea4af9e-fd59-4691-b61c-1fc1ff3e309e_KNW3G-22YD2-7QKQJ-2RF2X-H6%f%F8M_ServerDatacenterEval_2019",
    "1d02774d-66ab-4c57-8b14-e254fdce09d4_PK7JN-24236-FH7JP-V792F-37%f%CYR_ServerDatacenterEval_2021",
    "96794a98-097f-42fe-8f28-2c38ea115229_M4RNW-CRTHF-TY7BG-DDHG6-J2%f%T92_ServerDatacenterEval_2025",
    "38d172c7-36b3-4e4b-b435-fd0b06b95c6e_RNFGD-WFFQR-XQ8BG-K7QQK-GJ%f%CP9_ServerStandardEval_2012",
    "4fc45a88-26b5-4cf9-9eef-769ee3f0a016_79M8M-N36BX-8YGJY-2G9KP-3Y%f%GPC_ServerStandardEval_2012R2",
    "9dfa8ec0-7665-4b9d-b2cb-bfc2dc37c9f4_9PBKX-4NHGT-QWV4C-4JD94-TV%f%KQ6_ServerStandardEval_2016",
    "7783a126-c108-4cf7-b59f-13c78c7a7337_J4WNC-H9BG3-6XRX4-3XD8K-Y7%f%XRX_ServerStandardEval_2019",
    "c1a197b6-ba5e-4394-b9bf-b659a6c1b873_7PBJM-MNVPD-MBQD7-TYTY4-W8%f%JDY_ServerStandardEval_2021",
    "753c53a2-4274-4339-8c2e-f66c0b9646c5_YPBVM-HFNWQ-CTF9M-FR4RR-7H%f%9YG_ServerStandardEval_2025",
    "0de5ff31-2d62-4912-b1a8-3ea01d2461fd_3CKBN-3GJ8X-7YT4X-D8DDC-D6%f%69B_ServerStorageStandardEval_2012",
    "fb08f53a-e597-40dc-9f08-8bbf99f19b92_NCJ6J-J23VR-DBYB3-QQBJF-W8%f%CP7_ServerStorageWorkgroupEval_2012",
    "73111121-5638-40f6-bc11-f1d7b0d64300_NPPR9-FWDCX-D2C8J-H872K-2Y%f%T43___4_Enterprise",
    "e272e3e2-732f-4c65-a8f0-484747d0d947_DPH2V-TTNVB-4X9Q3-TJR4H-KH%f%JW4__27_EnterpriseN",
    "2de67392-b7a7-462a-b1ca-108dd189f588_W269N-WFGWX-YVC9B-4J6C9-T8%f%3GX__48_Professional",
    "a80b5abf-76ad-428b-b05d-a47d2dffeebf_MH37W-N47XK-V7XM9-C7227-GC%f%QG9__49_ProfessionalN",
    "7b9e1751-a8da-4f75-9560-5fadfe3d8e38_3KHY7-WNT83-DGQKR-F7HPR-84%f%4BM__98_CoreN",
    "a9107544-f4a0-4053-a96a-1479abdef912_PVMJN-6DFY6-9CCP6-7BKTT-D3%f%WVR__99_CoreCountrySpecific",
    "cd918a57-a41b-4c82-8dce-1a538e221a83_7HNRX-D7KGG-3K4RQ-4WPJ4-YT%f%DFH_100_CoreSingleLanguage",
    "58e97c99-f377-4ef1-81d5-4ad5522b5fd8_TX9XD-98N7V-6WMQ6-BX7FG-H8%f%Q99_101_Core",
    "e0c42288-980c-4788-a014-c080d2e1926e_NW6C2-QMPVW-D7KKK-3GKT6-VC%f%FB2_121_Education",
    "3c102355-d027-42c6-ad23-2e7ef8a02585_2WH4N-8QGBV-H22JP-CT43Q-MD%f%WWJ_122_EducationN",
    "32d2fab3-e4a8-42c2-923b-4bf4fd13e6ee_M7XTQ-FN8P6-TTKYV-9D4CC-J4%f%62D_125_EnterpriseS_RS5,VB,Ge",
    "2d5a5a60-3040-48bf-beb0-fcd770c20ce0_DCPHK-NFMTC-H88MJ-PFHPY-QJ%f%4BJ_125_EnterpriseS_RS1",
    "7b51a46c-0c04-4e8f-9af4-8496cca90d5e_WNMTR-4C88C-JK8YV-HQ7T2-76%f%DF9_125_EnterpriseS_TH1",
    "7103a333-b8c8-49cc-93ce-d37c09687f92_92NFX-8DJQP-P6BBQ-THF9C-7C%f%G2H_126_EnterpriseSN_RS5,VB,Ge",
    "9f776d83-7156-45b2-8a5c-359b9c9f22a3_QFFDN-GRT3P-VKWWX-X7T3R-8B%f%639_126_EnterpriseSN_RS1",
    "87b838b7-41b6-4590-8318-5797951d8529_2F77B-TNFGY-69QQF-B8YKP-D6%f%9TJ_126_EnterpriseSN_TH1",
    "82bbc092-bc50-4e16-8e18-b74fc486aec3_NRG8B-VKK3Q-CXVCJ-9G2XF-6Q%f%84J_161_ProfessionalWorkstation",
    "4b1571d3-bafb-4b40-8087-a961be2caf65_9FNHH-K3HBT-3W4TD-6383H-6X%f%YWF_162_ProfessionalWorkstationN",
    "3f1afc82-f8ac-4f6c-8005-1d233e606eee_6TP4R-GNPTD-KYYHQ-7B7DP-J4%f%47Y_164_ProfessionalEducation",
    "5300b18c-2e33-4dc2-8291-47ffcec746dd_YVWGF-BXNMC-HTQYQ-CPQ99-66%f%QFC_165_ProfessionalEducationN",
    "e0b2d383-d112-413f-8a80-97f373a5820c_YYVX9-NTFWV-6MDM3-9PT4T-4M%f%68B_171_EnterpriseG",
    "e38454fb-41a4-4f59-a5dc-25080e354730_44RPN-FTY23-9VTTB-MP9BX-T8%f%4FV_172_EnterpriseGN",
    "ec868e65-fadf-4759-b23e-93fe37f2cc29_CPWHC-NT2C7-VYW78-DHDB2-PG%f%3GK_175_ServerRdsh_RS5",
    "e4db50ea-bda1-4566-b047-0ca50abc6f07_7NBT4-WGBQX-MP4H7-QXFF8-YP%f%3KX_175_ServerRdsh_RS3",
    "0df4f814-3f57-4b8b-9a9d-fddadcd69fac_NBTWJ-3DR69-3C4V8-C26MC-GQ%f%9M6_183_CloudE",
    "59eb965c-9150-42b7-a0ec-22151b9897c5_KBN8V-HFGQ4-MGXVD-347P6-PD%f%QGT_191_IoTEnterpriseS_VB,NI",
    "d30136fc-cb4b-416e-a23d-87207abc44a9_6XN7V-PCBDC-BDBRH-8DQY7-G6%f%R44_202_CloudEditionN",
    "ca7df2e3-5ea0-47b8-9ac1-b1be4d8edd69_37D7F-N49CB-WQR8W-TBJ73-FM%f%8RX_203_CloudEdition",
];

thread_local! {
    static KEYS: LazyCell<Vec<GUID>> = LazyCell::new(|| {
        RAW_KEYS
            .iter()
            .map(|raw_key| {
                raw_key
                    .split('_')
                    .next()
                    .unwrap_or_else(|| panic!("bad key {raw_key}"))
            })
            .map(|guid_str| GUID::try_from(guid_str).unwrap_or_else(|_| panic!("bad guid {guid_str}")))
            .collect()
    });
}
