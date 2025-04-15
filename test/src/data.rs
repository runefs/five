pub mod data {
    use std::collections::HashMap;

    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    /// User profile struct matching OpenID Connect standard claims
    /// See: https://openid.net/specs/openid-connect-core-1_0.html#StandardClaims
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UserProfile {
        // Required
        sub: String,
        
        // Optional standard claims
        name: Option<String>,
        given_name: Option<String>,
        family_name: Option<String>,
        middle_name: Option<String>,
        nickname: Option<String>,
        preferred_username: Option<String>,
        profile: Option<String>,
        picture: Option<String>,
        website: Option<String>,
        email: Option<String>,
        email_verified: Option<bool>,
        gender: Option<String>,
        birthdate: Option<String>,
        zoneinfo: Option<String>,
        locale: Option<String>,
        phone_number: Option<String>,
        phone_number_verified: Option<bool>,
        address: Option<Address>,
        updated_at: Option<chrono::DateTime<chrono::Utc>>,
        
        // Additional custom claims
        additional_claims: HashMap<String, serde_json::Value>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Address {
        formatted: Option<String>,
        street_address: Option<String>,
        locality: Option<String>,
        region: Option<String>,
        postal_code: Option<String>,
        country: Option<String>,
    }

    impl UserProfile {
        /// Creates a new UserProfile with required fields
        pub fn new(sub: String) -> Self {
            UserProfile {
                sub,
                name: None,
                given_name: None,
                family_name: None,
                middle_name: None,
                nickname: None,
                preferred_username: None,
                profile: None,
                picture: None,
                website: None,
                email: None,
                email_verified: None,
                gender: None,
                birthdate: None,
                zoneinfo: None,
                locale: None,
                phone_number: None,
                phone_number_verified: None,
                address: None,
                updated_at: None,
                additional_claims: HashMap::new(),
            }
        }

        // Getters for all fields
        pub fn sub(&self) -> &str {
            &self.sub
        }
        
        pub fn name(&self) -> Option<&str> {
            self.name.as_deref()
        }
        
        pub fn given_name(&self) -> Option<&str> {
            self.given_name.as_deref()
        }
        
        pub fn family_name(&self) -> Option<&str> {
            self.family_name.as_deref()
        }
        
        pub fn middle_name(&self) -> Option<&str> {
            self.middle_name.as_deref()
        }
        
        pub fn nickname(&self) -> Option<&str> {
            self.nickname.as_deref()
        }
        
        pub fn preferred_username(&self) -> Option<&str> {
            self.preferred_username.as_deref()
        }
        
        pub fn profile(&self) -> Option<&str> {
            self.profile.as_deref()
        }
        
        pub fn picture(&self) -> Option<&str> {
            self.picture.as_deref()
        }
        
        pub fn website(&self) -> Option<&str> {
            self.website.as_deref()
        }
        
        pub fn email(&self) -> Option<&str> {
            self.email.as_deref()
        }
        
        pub fn email_verified(&self) -> Option<bool> {
            self.email_verified
        }
        
        pub fn gender(&self) -> Option<&str> {
            self.gender.as_deref()
        }
        
        pub fn birthdate(&self) -> Option<&str> {
            self.birthdate.as_deref()
        }
        
        pub fn zoneinfo(&self) -> Option<&str> {
            self.zoneinfo.as_deref()
        }
        
        pub fn locale(&self) -> Option<&str> {
            self.locale.as_deref()
        }
        
        pub fn phone_number(&self) -> Option<&str> {
            self.phone_number.as_deref()
        }
        
        pub fn phone_number_verified(&self) -> Option<bool> {
            self.phone_number_verified
        }
        
        pub fn address(&self) -> Option<&Address> {
            self.address.as_ref()
        }
        
        pub fn updated_at(&self) -> Option<&DateTime<Utc>> {
            self.updated_at.as_ref()
        }
        
        pub fn get_claim(&self, name: &str) -> Option<&serde_json::Value> {
            self.additional_claims.get(name)
        }
        
        // Non-destructive mutation methods (with_*)
        pub fn with_name(self, name: String) -> Self {
            UserProfile {
                name: Some(name),
                ..self
            }
        }
        
        pub fn with_given_name(self, given_name: String) -> Self {
            UserProfile {
                given_name: Some(given_name),
                ..self
            }
        }
        
        pub fn with_family_name(self, family_name: String) -> Self {
            UserProfile {
                family_name: Some(family_name),
                ..self
            }
        }
        
        pub fn with_middle_name(self, middle_name: String) -> Self {
            UserProfile {
                middle_name: Some(middle_name),
                ..self
            }
        }
        
        pub fn with_nickname(self, nickname: String) -> Self {
            UserProfile {
                nickname: Some(nickname),
                ..self
            }
        }
        
        pub fn with_preferred_username(self, preferred_username: String) -> Self {
            UserProfile {
                preferred_username: Some(preferred_username),
                ..self
            }
        }
        
        pub fn with_profile(self, profile: String) -> Self {
            UserProfile {
                profile: Some(profile),
                ..self
            }
        }
        
        pub fn with_picture(self, picture: String) -> Self {
            UserProfile {
                picture: Some(picture),
                ..self
            }
        }
        
        pub fn with_website(self, website: String) -> Self {
            UserProfile {
                website: Some(website),
                ..self
            }
        }
        
        pub fn with_email(self, email: String) -> Self {
            UserProfile {
                email: Some(email),
                ..self
            }
        }
        
        pub fn with_email_verified(self, email_verified: bool) -> Self {
            UserProfile {
                email_verified: Some(email_verified),
                ..self
            }
        }
        
        pub fn with_gender(self, gender: String) -> Self {
            UserProfile {
                gender: Some(gender),
                ..self
            }
        }
        
        pub fn with_birthdate(self, birthdate: String) -> Self {
            UserProfile {
                birthdate: Some(birthdate),
                ..self
            }
        }
        
        pub fn with_zoneinfo(self, zoneinfo: String) -> Self {
            UserProfile {
                zoneinfo: Some(zoneinfo),
                ..self
            }
        }
        
        pub fn with_locale(self, locale: String) -> Self {
            UserProfile {
                locale: Some(locale),
                ..self
            }
        }
        
        pub fn with_phone_number(self, phone_number: String) -> Self {
            UserProfile {
                phone_number: Some(phone_number),
                ..self
            }
        }
        
        pub fn with_phone_number_verified(self, phone_number_verified: bool) -> Self {
            UserProfile {
                phone_number_verified: Some(phone_number_verified),
                ..self
            }
        }
        
        pub fn with_address(self, address: Address) -> Self {
            UserProfile {
                address: Some(address),
                ..self
            }
        }
        
        pub fn with_updated_at(self, updated_at: DateTime<Utc>) -> Self {
            UserProfile {
                updated_at: Some(updated_at),
                ..self
            }
        }
        
        pub fn with_claim(self, name: String, value: serde_json::Value) -> Self {
            let mut claims = self.additional_claims.clone();
            claims.insert(name, value);
            
            UserProfile {
                additional_claims: claims,
                ..self
            }
        }
    }

    impl Address {
        pub fn new() -> Self {
            Address {
                formatted: None,
                street_address: None,
                locality: None,
                region: None,
                postal_code: None,
                country: None,
            }
        }
        
        pub fn formatted(&self) -> Option<&str> {
            self.formatted.as_deref()
        }
        
        pub fn street_address(&self) -> Option<&str> {
            self.street_address.as_deref()
        }
        
        pub fn locality(&self) -> Option<&str> {
            self.locality.as_deref()
        }
        
        pub fn region(&self) -> Option<&str> {
            self.region.as_deref()
        }
        
        pub fn postal_code(&self) -> Option<&str> {
            self.postal_code.as_deref()
        }
        
        pub fn country(&self) -> Option<&str> {
            self.country.as_deref()
        }
        
        // Non-destructive mutation methods
        pub fn with_formatted(self, formatted: String) -> Self {
            Address {
                formatted: Some(formatted),
                ..self
            }
        }
        
        pub fn with_street_address(self, street_address: String) -> Self {
            Address {
                street_address: Some(street_address),
                ..self
            }
        }
        
        pub fn with_locality(self, locality: String) -> Self {
            Address {
                locality: Some(locality),
                ..self
            }
        }
        
        pub fn with_region(self, region: String) -> Self {
            Address {
                region: Some(region),
                ..self
            }
        }
        
        pub fn with_postal_code(self, postal_code: String) -> Self {
            Address {
                postal_code: Some(postal_code),
                ..self
            }
        }
        
        pub fn with_country(self, country: String) -> Self {
            Address {
                country: Some(country),
                ..self
            }
        }
    }

}