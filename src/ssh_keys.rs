use russh::keys::PublicKey;

pub fn parse_authorized_keys(authorized_keys: &str) -> Vec<PublicKey> {
    authorized_keys
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| {
            let mut pieces = line.split_whitespace();

            match (pieces.next(), pieces.next()) {
                (Some(_), Some(key)) => russh::keys::parse_public_key_base64(key).ok(),
                (Some(key), None) => russh::keys::parse_public_key_base64(key).ok(),
                _ => None,
            }
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_authorized_keys_str() {
        let authorized_keys = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQCmn8DzRfmWKPKcVEPdCFFQbpdY2qzv5RkBLSAg1jlbLjHJuIyUf/e5lWwcfrtMLwEd5Wl6lgoEWxb2qsgEz1776D2QhWiXjGmKWmUHZiKrluiGlxHhqFDFJrjh1sQcBI5jReGGN5k1W06FrcGKCocsJ82cQbwahYjTU9UjhCPA4Q98pp7WGM0hctTlrGChvnszxKEqmX+4szv1bMYxHthT5l7Uuy0PsNJzQjoSOQJCs6a8EH2NB1nnufhT/rGZg6vqqAifa+Y+olulrBsuD4x/rIN/+FtFphWk02/xIxPH/2sUWcIE1/NCRLwFDGMPE/RItiOG08oixdL3Wb+Juok4Po63mwiCXZFFstIu1tlzykf40msxagX9sysYi1J6NMNVmKYGRayJp+C4ablYe2mVmOyqiktSIdo+IDPXSzuaZ6UicpbuM1HuS3z/T1eFNpHcYmZTkfVDZe72zOpCUmVkLuMgHxuMrIq/JFFYoymuN/aDqDZ0N/9QMnxlPQcmO+8= test@test\n\
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQCmn8DzRfmWKPKcVEPdCFFQbpdY2qzv5RkBLSAg1jlbLjHJuIyUf/e5lWwcfrtMLwEd5Wl6lgoEWxb2qsgEz1776D2QhWiXjGmKWmUHZiKrluiGlxHhqFDFJrjh1sQcBI5jReGGN5k1W06FrcGKCocsJ82cQbwahYjTU9UjhCPA4Q98pp7WGM0hctTlrGChvnszxKEqmX+4szv1bMYxHthT5l7Uuy0PsNJzQjoSOQJCs6a8EH2NB1nnufhT/rGZg6vqqAifa+Y+olulrBsuD4x/rIN/+FtFphWk02/xIxPH/2sUWcIE1/NCRLwFDGMPE/RItiOG08oixdL3Wb+Juok4Po63mwiCXZFFstIu1tlzykf40msxagX9sysYi1J6NMNVmKYGRayJp+C4ablYe2mVmOyqiktSIdo+IDPXSzuaZ6UicpbuM1HuS3z/T1eFNpHcYmZTkfVDZe72zOpCUmVkLuMgHxuMrIq/JFFYoymuN/aDqDZ0N/9QMnxlPQcmO+8=\n";

        let authorized_keys = parse_authorized_keys(authorized_keys);

        assert_eq!(2, authorized_keys.len());
    }

    #[test]
    fn test_parse_authorized_keys_str_with_whitespace() {
        let authorized_keys = "    \n \n     \n  \n";

        let authorized_keys = parse_authorized_keys(authorized_keys);

        assert_eq!(0, authorized_keys.len());
    }

    #[test]
    fn test_parse_authorized_keys_str_with_missing_piece() {
        let authorized_keys = "ssh-rsa";

        let authorized_keys = parse_authorized_keys(authorized_keys);

        assert_eq!(0, authorized_keys.len());
    }

    #[test]
    fn test_parse_authorized_keys_str_with_invalid_key() {
        let authorized_keys = "ssh-rsa invalid";

        let authorized_keys = parse_authorized_keys(authorized_keys);

        assert_eq!(0, authorized_keys.len());
    }
}
