#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingDynamicOps {
    id: &'static str,
    compress_maps: bool,
    calls: Vec<String>,
}

impl RecordingDynamicOps {
    pub fn new(id: &'static str, compress_maps: bool) -> Self {
        Self {
            id,
            compress_maps,
            calls: Vec::new(),
        }
    }

    pub fn id(&self) -> &'static str {
        self.id
    }

    pub fn calls(&self) -> &[String] {
        &self.calls
    }

    fn forward(&mut self, method: &str) -> String {
        self.calls.push(method.to_string());
        format!("{}:{method}", self.id)
    }

    fn forward_with(&mut self, method: &str, value: impl AsRef<str>) -> String {
        self.calls
            .push(format!("{method}:{}", value.as_ref()));
        format!("{}:{method}:{}", self.id, value.as_ref())
    }

    fn compress_maps(&mut self) -> bool {
        self.calls.push("compressMaps".to_string());
        self.compress_maps
    }
}

#[derive(Debug)]
pub struct DelegatingOpsModel {
    delegate: RecordingDynamicOps,
}

impl DelegatingOpsModel {
    pub fn new(delegate: RecordingDynamicOps) -> Self {
        Self { delegate }
    }

    pub fn delegate(&self) -> &RecordingDynamicOps {
        &self.delegate
    }

    pub fn ops_id(&self) -> &'static str {
        "delegating"
    }

    pub fn empty(&mut self) -> String {
        self.delegate.forward("empty")
    }

    pub fn empty_map(&mut self) -> String {
        self.delegate.forward("emptyMap")
    }

    pub fn empty_list(&mut self) -> String {
        self.delegate.forward("emptyList")
    }

    pub fn convert_to(&mut self, out_ops_is_delegate: bool, input: impl Into<String>) -> String {
        let input = input.into();
        if out_ops_is_delegate {
            input
        } else {
            self.delegate.forward_with("convertTo", input)
        }
    }

    pub fn get_number_value(&mut self) -> String {
        self.delegate.forward("getNumberValue")
    }

    pub fn create_numeric(&mut self, value: impl AsRef<str>) -> String {
        self.delegate.forward_with("createNumeric", value)
    }

    pub fn create_byte(&mut self, value: impl AsRef<str>) -> String {
        self.delegate.forward_with("createByte", value)
    }

    pub fn create_short(&mut self, value: impl AsRef<str>) -> String {
        self.delegate.forward_with("createShort", value)
    }

    pub fn create_int(&mut self, value: impl AsRef<str>) -> String {
        self.delegate.forward_with("createInt", value)
    }

    pub fn create_long(&mut self, value: impl AsRef<str>) -> String {
        self.delegate.forward_with("createLong", value)
    }

    pub fn create_float(&mut self, value: impl AsRef<str>) -> String {
        self.delegate.forward_with("createFloat", value)
    }

    pub fn create_double(&mut self, value: impl AsRef<str>) -> String {
        self.delegate.forward_with("createDouble", value)
    }

    pub fn get_boolean_value(&mut self) -> String {
        self.delegate.forward("getBooleanValue")
    }

    pub fn create_boolean(&mut self, value: impl AsRef<str>) -> String {
        self.delegate.forward_with("createBoolean", value)
    }

    pub fn get_string_value(&mut self) -> String {
        self.delegate.forward("getStringValue")
    }

    pub fn create_string(&mut self, value: impl AsRef<str>) -> String {
        self.delegate.forward_with("createString", value)
    }

    pub fn merge_to_list(&mut self) -> String {
        self.delegate.forward("mergeToList")
    }

    pub fn merge_to_map(&mut self) -> String {
        self.delegate.forward("mergeToMap")
    }

    pub fn merge_to_primitive(&mut self) -> String {
        self.delegate.forward("mergeToPrimitive")
    }

    pub fn get_map_values(&mut self) -> String {
        self.delegate.forward("getMapValues")
    }

    pub fn get_map_entries(&mut self) -> String {
        self.delegate.forward("getMapEntries")
    }

    pub fn create_map(&mut self) -> String {
        self.delegate.forward("createMap")
    }

    pub fn get_map(&mut self) -> String {
        self.delegate.forward("getMap")
    }

    pub fn get_stream(&mut self) -> String {
        self.delegate.forward("getStream")
    }

    pub fn get_list(&mut self) -> String {
        self.delegate.forward("getList")
    }

    pub fn create_list(&mut self) -> String {
        self.delegate.forward("createList")
    }

    pub fn get_byte_buffer(&mut self) -> String {
        self.delegate.forward("getByteBuffer")
    }

    pub fn create_byte_list(&mut self) -> String {
        self.delegate.forward("createByteList")
    }

    pub fn get_int_stream(&mut self) -> String {
        self.delegate.forward("getIntStream")
    }

    pub fn create_int_list(&mut self) -> String {
        self.delegate.forward("createIntList")
    }

    pub fn get_long_stream(&mut self) -> String {
        self.delegate.forward("getLongStream")
    }

    pub fn create_long_list(&mut self) -> String {
        self.delegate.forward("createLongList")
    }

    pub fn remove(&mut self, key: impl AsRef<str>) -> String {
        self.delegate.forward_with("remove", key)
    }

    pub fn compress_maps(&mut self) -> bool {
        self.delegate.compress_maps()
    }

    pub fn list_builder(&self, original: RecordingListBuilder) -> DelegateListBuilderModel {
        DelegateListBuilderModel::new(self.ops_id(), original)
    }

    pub fn map_builder(&self, original: RecordingRecordBuilder) -> DelegateRecordBuilderModel {
        DelegateRecordBuilderModel::new(self.ops_id(), original)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuilderReturn {
    Wrapper,
    Original,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecordingListBuilder {
    calls: Vec<String>,
}

impl RecordingListBuilder {
    pub fn calls(&self) -> &[String] {
        &self.calls
    }

    fn push(&mut self, call: impl Into<String>) {
        self.calls.push(call.into());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegateListBuilderModel {
    ops_id: &'static str,
    original: RecordingListBuilder,
}

impl DelegateListBuilderModel {
    fn new(ops_id: &'static str, original: RecordingListBuilder) -> Self {
        Self { ops_id, original }
    }

    pub fn ops(&self) -> &'static str {
        self.ops_id
    }

    pub fn original(&self) -> &RecordingListBuilder {
        &self.original
    }

    pub fn build(&mut self, prefix: impl AsRef<str>) -> String {
        self.original
            .push(format!("build:{}", prefix.as_ref()));
        format!("built:{}", prefix.as_ref())
    }

    pub fn add(&mut self, value: impl AsRef<str>) -> BuilderReturn {
        self.original.push(format!("add:{}", value.as_ref()));
        BuilderReturn::Wrapper
    }

    pub fn add_result(&mut self, value: impl AsRef<str>) -> BuilderReturn {
        self.original
            .push(format!("addDataResult:{}", value.as_ref()));
        BuilderReturn::Wrapper
    }

    pub fn add_encoded(&mut self, value: impl AsRef<str>) -> BuilderReturn {
        self.original.push(format!(
            "addEncoded:ops={}:value={}:prefix=empty",
            self.ops_id,
            value.as_ref()
        ));
        BuilderReturn::Wrapper
    }

    pub fn add_all_encoded(&mut self, values: &[&str]) -> BuilderReturn {
        for value in values {
            self.original.push(format!(
                "addEncoded:ops={}:value={value}:prefix=empty",
                self.ops_id
            ));
        }
        BuilderReturn::Wrapper
    }

    pub fn with_errors_from(&mut self, result: impl AsRef<str>) -> BuilderReturn {
        self.original
            .push(format!("withErrorsFrom:{}", result.as_ref()));
        BuilderReturn::Wrapper
    }

    pub fn map_error(&mut self, mapped: impl AsRef<str>) -> BuilderReturn {
        self.original
            .push(format!("mapError:{}", mapped.as_ref()));
        BuilderReturn::Wrapper
    }

    pub fn build_result(&mut self, prefix: impl AsRef<str>) -> String {
        self.original
            .push(format!("buildDataResult:{}", prefix.as_ref()));
        format!("built-result:{}", prefix.as_ref())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecordingRecordBuilder {
    calls: Vec<String>,
}

impl RecordingRecordBuilder {
    pub fn calls(&self) -> &[String] {
        &self.calls
    }

    fn push(&mut self, call: impl Into<String>) {
        self.calls.push(call.into());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegateRecordBuilderModel {
    ops_id: &'static str,
    original: RecordingRecordBuilder,
}

impl DelegateRecordBuilderModel {
    fn new(ops_id: &'static str, original: RecordingRecordBuilder) -> Self {
        Self { ops_id, original }
    }

    pub fn ops(&self) -> &'static str {
        self.ops_id
    }

    pub fn original(&self) -> &RecordingRecordBuilder {
        &self.original
    }

    pub fn add(&mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> BuilderReturn {
        self.original
            .push(format!("add:{}={}", key.as_ref(), value.as_ref()));
        BuilderReturn::Wrapper
    }

    pub fn add_value_result(
        &mut self,
        key: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> BuilderReturn {
        self.original
            .push(format!("addValueDataResult:{}={}", key.as_ref(), value.as_ref()));
        BuilderReturn::Wrapper
    }

    pub fn add_key_value_result(
        &mut self,
        key: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> BuilderReturn {
        self.original.push(format!(
            "addKeyValueDataResult:{}={}",
            key.as_ref(),
            value.as_ref()
        ));
        BuilderReturn::Wrapper
    }

    pub fn add_string_key(&mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> BuilderReturn {
        self.original
            .push(format!("addStringKey:{}={}", key.as_ref(), value.as_ref()));
        BuilderReturn::Wrapper
    }

    pub fn add_string_key_result(
        &mut self,
        key: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> BuilderReturn {
        self.original.push(format!(
            "addStringKeyDataResult:{}={}",
            key.as_ref(),
            value.as_ref()
        ));
        BuilderReturn::Wrapper
    }

    pub fn add_encoded(&mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> BuilderReturn {
        self.original.push(format!(
            "addEncoded:ops={}:{}={}",
            self.ops_id,
            key.as_ref(),
            value.as_ref()
        ));
        BuilderReturn::Original
    }

    pub fn with_errors_from(&mut self, result: impl AsRef<str>) -> BuilderReturn {
        self.original
            .push(format!("withErrorsFrom:{}", result.as_ref()));
        BuilderReturn::Wrapper
    }

    pub fn set_lifecycle(&mut self, lifecycle: impl AsRef<str>) -> BuilderReturn {
        self.original
            .push(format!("setLifecycle:{}", lifecycle.as_ref()));
        BuilderReturn::Wrapper
    }

    pub fn map_error(&mut self, mapped: impl AsRef<str>) -> BuilderReturn {
        self.original
            .push(format!("mapError:{}", mapped.as_ref()));
        BuilderReturn::Wrapper
    }

    pub fn build(&mut self, prefix: impl AsRef<str>) -> String {
        self.original
            .push(format!("build:{}", prefix.as_ref()));
        format!("built:{}", prefix.as_ref())
    }

    pub fn build_result(&mut self, prefix: impl AsRef<str>) -> String {
        self.original
            .push(format!("buildDataResult:{}", prefix.as_ref()));
        format!("built-result:{}", prefix.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DELEGATING_OPS_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/resources/DelegatingOps.java");

    #[test]
    fn dynamic_ops_methods_forward_to_delegate_except_convert_to_same_delegate() {
        assert_java_contains(
            DELEGATING_OPS_JAVA,
            &[
                "public abstract class DelegatingOps<T> implements DynamicOps<T>",
                "protected final DynamicOps<T> delegate;",
                "protected DelegatingOps(final DynamicOps<T> delegate)",
                "return (U)(Objects.equals(outOps, this.delegate) ? input : this.delegate.convertTo(outOps, input));",
                "return this.delegate.getNumberValue(input);",
                "return (T)this.delegate.createString(value);",
                "return this.delegate.mergeToMap(map, values);",
                "return (T)this.delegate.remove(input, key);",
                "return this.delegate.compressMaps();",
            ],
        );

        let mut ops = DelegatingOpsModel::new(RecordingDynamicOps::new("delegate", true));
        assert_eq!(ops.empty(), "delegate:empty");
        assert_eq!(ops.create_int("4"), "delegate:createInt:4");
        assert_eq!(ops.create_string("stone"), "delegate:createString:stone");
        assert_eq!(ops.merge_to_map(), "delegate:mergeToMap");
        assert_eq!(ops.remove("foo"), "delegate:remove:foo");
        assert!(ops.compress_maps());

        assert_eq!(ops.convert_to(true, "same-value"), "same-value");
        assert_eq!(
            ops.convert_to(false, "foreign-value"),
            "delegate:convertTo:foreign-value"
        );
        assert_eq!(
            ops.delegate().calls(),
            &[
                "empty",
                "createInt:4",
                "createString:stone",
                "mergeToMap",
                "remove:foo",
                "compressMaps",
                "convertTo:foreign-value"
            ]
        );
    }

    #[test]
    fn list_builder_wraps_delegate_builder_and_reports_outer_ops() {
        assert_java_contains(
            DELEGATING_OPS_JAVA,
            &[
                "public ListBuilder<T> listBuilder()",
                "return new DelegatingOps.DelegateListBuilder(this.delegate.listBuilder());",
                "public DynamicOps<T> ops()",
                "return DelegatingOps.this;",
                "this.original.add(encoder.encodeStart(this.ops(), value));",
                "values.forEach(v -> this.original.add(encoder.encode(v, this.ops(), this.ops().empty())));",
                "this.original.withErrorsFrom(result);",
                "return this.original.build(prefix);",
            ],
        );

        let ops = DelegatingOpsModel::new(RecordingDynamicOps::new("delegate", false));
        let mut builder = ops.list_builder(RecordingListBuilder::default());
        assert_eq!(builder.ops(), "delegating");
        assert_eq!(builder.add("direct"), BuilderReturn::Wrapper);
        assert_eq!(builder.add_result("result"), BuilderReturn::Wrapper);
        assert_eq!(builder.add_encoded("encoded"), BuilderReturn::Wrapper);
        assert_eq!(builder.add_all_encoded(&["a", "b"]), BuilderReturn::Wrapper);
        assert_eq!(builder.with_errors_from("err"), BuilderReturn::Wrapper);
        assert_eq!(builder.map_error("mapped"), BuilderReturn::Wrapper);
        assert_eq!(builder.build("prefix"), "built:prefix");
        assert_eq!(builder.build_result("result-prefix"), "built-result:result-prefix");
        assert_eq!(
            builder.original().calls(),
            &[
                "add:direct",
                "addDataResult:result",
                "addEncoded:ops=delegating:value=encoded:prefix=empty",
                "addEncoded:ops=delegating:value=a:prefix=empty",
                "addEncoded:ops=delegating:value=b:prefix=empty",
                "withErrorsFrom:err",
                "mapError:mapped",
                "build:prefix",
                "buildDataResult:result-prefix"
            ]
        );
    }

    #[test]
    fn record_builder_wraps_delegate_builder_and_preserves_encoded_return_shape() {
        assert_java_contains(
            DELEGATING_OPS_JAVA,
            &[
                "public RecordBuilder<T> mapBuilder()",
                "return new DelegatingOps.DelegateRecordBuilder(this.delegate.mapBuilder());",
                "return DelegatingOps.this;",
                "this.original.add(key, value);",
                "this.original.add(key, value);",
                "return this.original.add(key, encoder.encodeStart(this.ops(), value));",
                "this.original.setLifecycle(lifecycle);",
                "return this.original.build(prefix);",
            ],
        );

        let ops = DelegatingOpsModel::new(RecordingDynamicOps::new("delegate", false));
        let mut builder = ops.map_builder(RecordingRecordBuilder::default());
        assert_eq!(builder.ops(), "delegating");
        assert_eq!(builder.add("key", "value"), BuilderReturn::Wrapper);
        assert_eq!(builder.add_value_result("key", "result"), BuilderReturn::Wrapper);
        assert_eq!(
            builder.add_key_value_result("key-result", "value-result"),
            BuilderReturn::Wrapper
        );
        assert_eq!(builder.add_string_key("name", "stone"), BuilderReturn::Wrapper);
        assert_eq!(
            builder.add_string_key_result("name", "result"),
            BuilderReturn::Wrapper
        );
        assert_eq!(builder.add_encoded("encoded", "value"), BuilderReturn::Original);
        assert_eq!(builder.with_errors_from("err"), BuilderReturn::Wrapper);
        assert_eq!(builder.set_lifecycle("stable"), BuilderReturn::Wrapper);
        assert_eq!(builder.map_error("mapped"), BuilderReturn::Wrapper);
        assert_eq!(builder.build("prefix"), "built:prefix");
        assert_eq!(builder.build_result("result-prefix"), "built-result:result-prefix");
        assert_eq!(
            builder.original().calls(),
            &[
                "add:key=value",
                "addValueDataResult:key=result",
                "addKeyValueDataResult:key-result=value-result",
                "addStringKey:name=stone",
                "addStringKeyDataResult:name=result",
                "addEncoded:ops=delegating:encoded=value",
                "withErrorsFrom:err",
                "setLifecycle:stable",
                "mapError:mapped",
                "build:prefix",
                "buildDataResult:result-prefix"
            ]
        );
    }

    fn assert_java_contains(source: &str, sentinels: &[&str]) {
        if source.is_empty() {
            return;
        }
        for sentinel in sentinels {
            assert!(
                source.contains(sentinel),
                "missing Java source sentinel {sentinel}"
            );
        }
    }
}
