use canfund::api::cmc::GetIcpXdrResult;
use ic_ledger_types::MAINNET_CYCLES_MINTING_CANISTER_ID;

const CYCLES_PER_XDR: u128 = 1_000_000_000_000; // 1 trillion cycles per XDR

pub struct Cmc;

impl Cmc {
    pub async fn cycles_to_icp(
        cycles: u128
    ) -> Result<u128, String> {
        let price = Self::get_icp_xdr_rate().await?;

        Ok(
            Self::calculate_icp_amount(
                cycles, 
                price
            )
        )
    }

    async fn get_icp_xdr_rate(
    ) -> Result<u128, String> {
        let res = ic_cdk::call::<(), (GetIcpXdrResult,)>(
            MAINNET_CYCLES_MINTING_CANISTER_ID,
            "get_icp_xdr_conversion_rate",
            (),
        )
        .await
            .map_err(|e| e.1)?;

        Ok(res.0.data.xdr_permyriad_per_icp as _)
    }

    fn calculate_icp_amount(
        cycles_amount: u128, 
        xdr_permyriad_per_icp: u128
    ) -> u128 {
        let cycles_per_icp = xdr_permyriad_per_icp * CYCLES_PER_XDR / 10_000u128;
        cycles_amount * 100_000_000u128 / cycles_per_icp
    }
}