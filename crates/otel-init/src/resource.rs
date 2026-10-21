use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;

/// Creates a resource using SDK defaults and additional attributes.
///
/// This function builds an OpenTelemetry resource that identifies your service
/// and includes any additional attributes you want to track.
/// The SDK detects environment attributes and supplies a default service name.
/// Explicit attributes override detected attributes, including `service.name`.
///
/// # Arguments
///
/// * `attributes` - Additional key-value pairs to include in the resource
///
/// # Examples
///
/// ```rust
/// use otel_init::get_resource;
/// use opentelemetry::KeyValue;
///
/// let resource = get_resource(&[KeyValue::new("environment", "production")]);
/// ```
pub fn get_resource(attributes: &[KeyValue]) -> Resource {
    Resource::builder()
        .with_attributes(attributes.iter().cloned())
        .build()
}

#[cfg(test)]
mod tests {
    use opentelemetry::{Key, KeyValue, Value};
    use opentelemetry_sdk::Resource;

    use super::get_resource;

    #[test]
    fn test_get_resource_uses_sdk_defaults() {
        assert_eq!(get_resource(&[]), Resource::builder().build());
    }

    #[test]
    fn test_get_resource_with_explicit_attributes() {
        let attributes = [
            KeyValue::new("service.name", "test-service"),
            KeyValue::new("env", "test"),
            KeyValue::new("version", "1.0.0"),
        ];

        let resource = get_resource(&attributes);

        assert_eq!(
            resource.get(&Key::new("service.name")),
            Some(Value::String("test-service".into()))
        );

        assert_eq!(
            resource.get(&Key::new("env")),
            Some(Value::String("test".into()))
        );

        assert_eq!(
            resource.get(&Key::new("version")),
            Some(Value::String("1.0.0".into()))
        );
    }
}
