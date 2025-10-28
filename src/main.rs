use freya::prelude::*;

fn main() {
    // **Start** your app by specifying the root component, title and size
    launch_with_props(app, "Counter", (400.0, 350.0));
}

fn app() -> Element {
   // Define a **state**
   let mut count = use_signal(|| 0);

   // Declare the **UI**
   rsx!(
    rect {
        height: "100%",
        width: "100%",
        background: "rgb(35, 35, 35)",
        color: "white",
        padding: "12",
        onclick: move |_| count += 1, // **Update** the state on click events
        label { "Click to increase -> {count}" } // Display the **state**
    }
)
}