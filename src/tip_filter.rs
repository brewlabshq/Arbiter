use solana_sdk::pubkey::Pubkey;
use solana_system_interface::instruction::SystemInstruction;
use solana_transaction::versioned::VersionedTransaction;
use std::str::FromStr;
use bincode::deserialize;

pub const JITO_TIP_ACCOUNTS: [&str; 8] = [
    "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
    "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
    "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
    "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49",
    "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
    "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
    "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
    "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TipInfo {
    pub lamports: u64,
    pub tip_account: String,
}

fn is_tip_account(account_bytes: &[u8; 32]) -> Option<&'static str> {
    for tip_account in JITO_TIP_ACCOUNTS {
        let tip_pubkey = Pubkey::from_str(tip_account).ok()?;
        if tip_pubkey.to_bytes() == *account_bytes {
            return Some(tip_account);
        }
    }
    None
}

pub fn extract_tip(tx: &VersionedTransaction) -> Option<TipInfo> {
    let message = &tx.message;
    let account_keys = message.static_account_keys();
    let system_program = Pubkey::from_str("11111111111111111111111111111111").unwrap();

    // println!("message: {:?}", message);
    for instruction in message.instructions() {
        // println!("instruction: {:?}", instruction);
        let program_id_index = instruction.program_id_index as usize;
        if program_id_index >= account_keys.len() {
            continue;
        }

        // println!("program_id_index: {:?}", program_id_index);
        let program_id_bytes: &[u8] = account_keys[program_id_index].as_ref();
        if program_id_bytes != system_program.as_ref() {
            continue;
        }

        println!("instruction.data: {:?}", instruction.data);
        let Ok(system_ix) = deserialize::<SystemInstruction>(&instruction.data) else {
            continue;
        };
        println!("system_ix: {:?}", system_ix);

        if let SystemInstruction::Transfer { lamports } = system_ix {
            if instruction.accounts.len() < 2 {
                continue;
            }
            println!("instruction.accounts: {:?}", instruction.accounts);

            let dest_index = instruction.accounts[1] as usize;
            if dest_index >= account_keys.len() {
                continue;
            }
            println!("dest_index: {:?}", dest_index);

            let dest_bytes: &[u8] = account_keys[dest_index].as_ref();
            println!("dest_bytes: {:?}", dest_bytes);
            let destination: [u8; 32] = match dest_bytes.try_into() {
                Ok(arr) => arr,
                Err(_) => continue,
            };
            println!("destination: {:?}", destination);

            if let Some(tip_account) = is_tip_account(&destination) {
                println!("tip_account: {:?}", tip_account);
                return Some(TipInfo {
                    lamports,
                    tip_account: tip_account.to_string(),
                });
            }
        }
    }

    tracing::debug!("No tip found in this transaction");
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // bundle ID -> 0a92577daebadf2214cb91bf235c85bb6ca6be15ec13c61acaef3f8ced15b4d1
    // const MAINNET_TX_BASE64: &str = "A/5aeh7m9fliV5l2NMW6Hh6lAmGdH3Sf5Y1aFg3sovrzs19W7xCNhh5jWX+Gxrp6l1+Y+PKKLOmDkyfW1VNFrg/v7iubgZpeBrvAdlNlEMVUyLGR1DLmPOscaL9PUKZ0+OAWMsIEFi9pmJI9Vbh9gR8uMNVp16iOTgXWvBHCPIQCZosLlFeyYV8PYcCeRGsa1l3KKpfNrfDkkxArPqbw6hYERpCw4Mk9GdQgd6SVWtC2p5rM2I1zk+6vtMQlC8llBIADAAIO8fpNi9xOXchA3rcC8shDA4YGoun3J4aTeqgnmnZ0th0ONi+hCOad0eoPx1jP2kHlW72Ye8U9ULevdJlt3/hrKznji/lyhpN6JMMWOJ17aBWB9c2eSMQWx1BOjVdVTYi7d9gPK5XWPM6uXrt+BB+pm9s9R0B9gxTHDnLnELh23V9YpoU+l0+EERjop4srL+m9gTew8gdpKvShJSOLwo/Nbtm3XNtcTUVBUo+cqWrFgYHdzbDbOdik8Pe69r34TPgOVfNRlAE+aWaelzriE3SJk1rHXQjE3rySDBR27dH3tYqwA6caU995b+AlcKJ/nUnvMhZjyw9/uI1Qk6Pp7w09YA0mpH92G94UUPv58qwAM/7B+lhqyO+cXLF2l7bbHNQVCvhYHoP+wfhHen2v1Z7z/QFHAPUum9CCO3GGelhZ9ElxAGR5S/4q7GXQKqsHjZWJM5jYn/X4+8KZ8OxMDhN+yHXhyPxU1fkaYvf3Xy03J1Pi8WZKSqTrOWbVVdkf+G/ABt324ddloZPZy+FGzut5rBy0he1fWzeROoz1hX7/AKkAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAICPPc3XuOj/jF4Rpv03sLSUd2bIhoe/Ba+aISt4KkuHDAwDAwQBCQhYJqFSAAAAAAwDAwABAQkMAwUGAQkIybhpHQAAAAAMAwUAAQEJDQIBAAwCAAAAVSSfAAAAAAAMAwcIAgkIXiHjYgAAAAAMAwcAAgEJDAMJBAIJCDl82EwAAAAADAMJAAIBCQwDCgsCCQicClOKAAAAAAwDCgACAQkNAgIADAIAAACwXXYAAAAAAAA=";
    // const MAINNET_TX_BASE64: &str = "A4ohfPqvdB6iDu+RdJQoTaEDze07vjQHTD3FQsS4Hq5ZaRIZMPKivxINqwkA7NeVfHLrjrh1OhZcWFTZDY8oVAGQcBf4pvUQBNdGYqodkM8lAx7q16alzdOMayEbHwhMJtStG6I6veJbj3DRhfC1K5rSqC7g/hdiypZaIU9OZfsJ7h3PFvpvmISsFQ8eh8JTPwVfgh+fY/FwBhsj4UmlD31CL3mCAQlkaTP+1vSsvGB3Py1F2FOM6ZKkDubJjalQC4ADAAIT8fpNi9xOXchA3rcC8shDA4YGoun3J4aTeqgnmnZ0th3ZalS+HxLmnS6TQeoHHmxVIX9ox4E+3REdXF36QX0vGpWHOnp3c2A0RSiVcVyzvkqTT+Tjs+BvHBhTDfv2420TOVX1ZtrttF5kcArtOtWH46hCgxm5yI0o4ZgTQcdTAAoNJqR/dhveFFD7+fKsADP+wfpYasjvnFyxdpe22xzUFX1xRwRncoR24my2l2B7wsF60TLmc0RExwBebGHDyBnYVfNRlAE+aWaelzriE3SJk1rHXQjE3rySDBR27dH3tYq8LYFDWBFPOEV4BLjK3fMWKt5PFA7OFPmFVG710DeRF+OYHNW/jrqXyIF2hcR6GxzPVLt9GZCHOLSs4e5MoNkT3WIQ/EQsfBEwKKZWYQMDyHhfAQTwL8S6MIUdeF4vwfHfox7ShEpacT0WsbRzs4PYuuTcB5hp3ZCzTuaTpv8fgfZExrDrrgHkQhFOBSeLUlbd9q6VXNQ89CFtc7WA3sQ75uIFPx3CCXvX43R7sA8kDZFF+Vnd3lHtawnisu/WaBxYpoU+l0+EERjop4srL+m9gTew8gdpKvShJSOLwo/NbqW81ZMXE0K5ReIIEL7xgxrY+rT39PmaNVAFbp7091CCdeHI/FTV+Rpi9/dfLTcnU+LxZkpKpOs5ZtVV2R/4b8AaKMYIiuh1EbGvYbridieg9yZM0HQjYSGqQJYIXw4Yiwbd9uHXZaGT2cvhRs7reawctIXtX1s3kTqM9YV+/wCpAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAjz3N17jo/4xeEab9N7C0lHdmyIaHvwWvmiEreCpLhxIRAwMEAQkItZ/WdQAAAAARAwMAAQEJEQMFBgEJCFhuOiQAAAAAEQMFAAEBCREDBwgBCQgeEY0xAAAAABEDBwABAQkRAwkKAQkI32//OgAAAAARAwkAAQEJEgIBAAwCAAAA1xBZAAAAAAARAwsGAgkI5o6bOAAAAAARAwsAAgEJEQMMDQIJCGcOrWwAAAAAEQMMAAIBCREDDg8CCQgxAVM5AAAAABEDDgACAQkRAxAIAgkIt4qaKQEAAAARAxAAAgEJEgICAAwCAAAAYVQuAAAAAAAA";
    // const MAINNET_TX_BASE64: &str = "AyxRfjs9CFIR7l4pntwvOfda9Dxy3BPLh9q7HP2YUpzwdWCWhoNWjuGSBRH8wtVRgL5ddJ82tqFNSaZuTpa5AwonJag50zxGHq/55EzKGtygznl/5QCYp4yA3xXAb/ZQwL8wwuS53kcijU2x+K8+oOE4Uss1suPKePD8ssrT2JQNJ8dtzsPf53qX2Jb7vYDFN3LsIXYKZKwJO5k/KPGUQ/8b8YBoUBpqu0If5XAwAq9iCa+7K4hksUNxQzIjap4HCYADAAIP8fpNi9xOXchA3rcC8shDA4YGoun3J4aTeqgnmnZ0th3zYKHuxDSTKQGodLpQfwVhM03qd67gkosrsfJYLW7SRXiBEn1d2XOfJ2m1E1RAP6hSEPLn9YrX+erXHmOH75QopjhRihdNYsFjfXEJ15PIG65wPX4aaDwRRUj6mI2SeZxV81GUAT5pZp6XOuITdImTWsddCMTevJIMFHbt0fe1ipiA8Ljp4nm6OltBcrm9pTq5HHC2tla0bw7WROjpDgck45gc1b+OupfIgXaFxHobHM9Uu30ZkIc4tKzh7kyg2RMSUlWe1p2Dd2ZAHj6ZmKrANOY0bNhfqTCHf+tjYDkrxXXhyPxU1fkaYvf3Xy03J1Pi8WZKSqTrOWbVVdkf+G/AYeHLtHXQDdn/Zz2jcPC6ymv+JYGPlSgdVS04u4exFPBp7johrshNoFLlmmhM7PTTwbovc4zEljuQ+j/xOM1LMFQY/7msziZs7ay+ASXXo1FH2hP28e5W97mP9Md+TW7lDSakf3Yb3hRQ+/nyrAAz/sH6WGrI75xcsXaXttsc1BUG3fbh12Whk9nL4UbO63msHLSF7V9bN5E6jPWFfv8AqQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgI89zde46P+MXhGm/TewtJR3ZsiGh78Fr5ohK3gqS4cODQMDBAEJCPA3+UwAAAAADQMDAAEBCQ0DBQYBCQjNH+kwAAAAAA0DBQABAQkOAgEADAIAAADBp6AAAAAAAA0DBwgCCQjtskJGAAAAAA0DBwACAQkNAwkEAgkIc8/edAAAAAANAwkAAgEJDQMKBgIJCD3gB3QAAAAADQMKAAIBCQ0DCwwCCQhpRsURAAAAAA0DCwACAQkOAgIADAIAAAAV31QAAAAAAAA="; 
    // const MAINNET_TX_BASE64: &str = "AyHdRfeCN9mO+r4fZo8MnNzbbPi9jQ/SUeqH2WAXQDBZu+cXWbHuwk9zNV8ncCcEUQKKUdC+KOK/WEGtRSLa+QjxY+0jBOLbfAWWt8I5fHm4OWPnBimUXIPSDgj1RMPt0XCky9LIkWod+8jKDkRZUTlbd7CModE/HDkRDzYCVqIHHbCgaVKGduIVvvhoEFS4wYRODHvv6y7oQXnhXgr/b2A9Dbh0bwYFD/6Y0kL6yi41fJBuqjmSlc1ab7V11GSVBYADAAIP8fpNi9xOXchA3rcC8shDA4YGoun3J4aTeqgnmnZ0th0Kv/D8QceFNudA7LYYQL0eYwS/5nP00jxGTwnumbi70adCYD8ZsOBv6AjBreS1HytihBHe4KyaljjpU1A8fVyip+dxfst4OIUWlpqsGSYcjm0yN6lhHeQxRSpTsPewixQNJqR/dhveFFD7+fKsADP+wfpYasjvnFyxdpe22xzUFelrJXJ1GLAGBA9GJyC+/ZDW5P2UpK9IPUaK0gBl+Uw545gc1b+OupfIgXaFxHobHM9Uu30ZkIc4tKzh7kyg2RNzCIzyl6mQJC80hRwrb3I30LOCLnObAiPZDlovVpMp/HXhyPxU1fkaYvf3Xy03J1Pi8WZKSqTrOWbVVdkf+G/ASkgztIZ52KwDL1z+DpiV3cMDXZLqHWTsTummBNj9rh67wKO0K4VfFERYcxfZ8pizf0+S77ebyHCi2Srk4MlirVXzUZQBPmlmnpc64hN0iZNax10IxN68kgwUdu3R97WKy8MVmCZ/CEblvsl4UK2VWdhMC0dzeVEyNRnaWoyb+sYG3fbh12Whk9nL4UbO63msHLSF7V9bN5E6jPWFfv8AqQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgI89zde46P+MXhGm/TewtJR3ZsiGh78Fr5ohK3gqS4cODQMDBAEJCC87fBUAAAAADQMDAAEBCQ0DBQYBCQhm17ljAAAAAA0DBQABAQkOAgEADAIAAAD1QJ4AAAAAAA0DBwgCCQhyludCAAAAAA0DBwACAQkNAwkEAgkIyymkfgAAAAANAwkAAgEJDQMKCwIJCN6EYiwAAAAADQMKAAIBCQ0DDAYCCQgilD5VAAAAAA0DDAACAQkOAgIADAIAAACtS1QAAAAAAAA=";
    const MAINNET_TX_BASE64: &str = "AzsIYOacDEvt/+hqqE5wZGoIlLRJ8NtJXJfyoTKO0SIv8Bbosqms092ab9PDQF5R6xrphYCX+VVKk8pyCI3zRAc1vPB4uxlsJrjmUe47kRy567lW2mNSBe4tQCFbHeKVdUcRHeYhsGU67flerIDW9dlYyi7L1Pr4oqcZhcarlGYJTPVod+rCecHI4A6P7DFTmZt/vZyaPWigP4x5TAYliL4IvvfdxEzSNJ9cI67QXQ8sngRZX9jFytT/jtFkkjSSCoADAAIO8fpNi9xOXchA3rcC8shDA4YGoun3J4aTeqgnmnZ0th0i/i75WLCyTiucRbQmMpLJ2KFCMfwKfAp2P3UW7v86/Xjz2E/Vmt5WaE3pMTN78xsK2b/tUATvfULN/OuXdzoEADUXiLqfYVeJt62ozUl6N2x+cIWY/Bb4ChQCutjeMGvfox7ShEpacT0WsbRzs4PYuuTcB5hp3ZCzTuaTpv8fgbVuVXy5KGsvGl8SYnbG87BsqReCMXtKaDwP4RLd94rfDSakf3Yb3hRQ+/nyrAAz/sH6WGrI75xcsXaXttsc1BVsv6S8K7Dwtb4NkuJNfb76bVo2Pff20mUTncKnh3A3ROOYHNW/jrqXyIF2hcR6GxzPVLt9GZCHOLSs4e5MoNkTIzQKBNbrLu4HzLwioIzCX8I0eml0g08IhNW1YL3B5ZdYpoU+l0+EERjop4srL+m9gTew8gdpKvShJSOLwo/NbrFODeVen7qGOW6/1UjP+MkgEerHt1uqmy2caob1oXFBBt324ddloZPZy+FGzut5rBy0he1fWzeROoz1hX7/AKkAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAICPPc3XuOj/jF4Rpv03sLSUd2bIhoe/Ba+aISt4KkuHCwwDAwQBCQjKCw1BAAAAAAwDAwABAQkNAgEADAIAAAChc8IAAAAAAAwDBQYCCQj5HJ6wAAAAAAwDBQACAQkMAwcIAgkIwW6JXAAAAAAMAwcAAgEJDAMJCgIJCDC/DCUAAAAADAMJAAIBCQ0CAgAMAgAAAAAodwAAAAAADQIACwwCAAAA6AMAAAAAAAAA";

    const EXPECTED_TIP_LAMPORTS: u64 = 1000;
    const EXPECTED_TIP_ACCOUNT: &str = "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY";

    fn decode_mainnet_tx(base64_tx: &str) -> VersionedTransaction {
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(base64_tx)
            .expect("Failed to decode base64");
        deserialize(&bytes).expect("Failed to deserialize transaction")
    }

    #[test]
    fn test_mainnet_transaction_with_tip() {
        let tx = decode_mainnet_tx(MAINNET_TX_BASE64);
        let tip_info = extract_tip(&tx);

        println!("tip_info: {:?}", tip_info);

        assert!(tip_info.is_some(), "No tip found in this transaction");

        let tip = tip_info.unwrap();
        assert_eq!(
            tip.lamports, EXPECTED_TIP_LAMPORTS,
            "Expected {} lamports, got {}",
            EXPECTED_TIP_LAMPORTS, tip.lamports
        );

        assert_eq!(
            tip.tip_account, EXPECTED_TIP_ACCOUNT,
            "Tip account mismatch"
        );

        println!(
            "SUCCESS: Found tip of {} lamports to {}",
            tip.lamports, tip.tip_account
        );
    }
}
