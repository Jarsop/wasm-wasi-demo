wit_bindgen::generate!({
    path: "../../wit",
    world: "publish",
    exports: {
        world: PublishComponent,
    },
});

struct PublishComponent;

impl Guest for PublishComponent {
    fn publish(mut payload: String) -> (StatusCode, Option<String>) {
        payload.push_str(" from wasm");
        let result = host::fetch(
            host::Method::Post,
            "http://localhost:8080/publish",
            Some(&payload),
        );
        result
    }
}
