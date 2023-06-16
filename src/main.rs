// Entry point for non-wasm
//NEXT STEP - FIGURE OUT HOW TO ROTATE MESHES
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    run().await;
}

use three_d::*;
use rand::Rng;
use web_sys::console;

struct Perceptron {
    weights: Vec<f64>,
    learning_rate: f64,
}

impl Perceptron {
    fn new(num_inputs: usize, learning_rate: f64) -> Self {
        let mut rng = rand::thread_rng();
        let weights = (0..=num_inputs).map(|_| rng.gen::<f64>()).collect();
        Perceptron {
            weights,
            learning_rate,
        }
    }

    fn predict(&self, inputs: &[f64]) -> i32 {
        let sum = self
            .weights
            .iter()
            .zip(inputs.iter())
            .map(|(&w, &x)| w * x)
            .sum::<f64>();
        if sum >= 0.0 {
            1
        } else {
            -1
        }
    }

    fn train(&mut self, inputs: &[&[f64]], targets: &[i32], max_epochs: usize) {
        for _ in 0..max_epochs {
            let mut error_count = 0;
            for (&input, &target) in inputs.iter().zip(targets.iter()) {
                let prediction = self.predict(input);
                let error = target - prediction;
                if error != 0 {
                    error_count += 1;
                    for (weight, &x) in self.weights.iter_mut().zip(input.iter()) {
                        *weight += self.learning_rate * (error as f64) * x;
                    }
                }
            }
            if error_count == 0 {
                break;
            }
        }
    }    
}

pub async fn run() {
    //Set up three-d context
    let window = Window::new(WindowSettings {
        title: "Multiple Shape Selection".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();

    //Set up camera and lighting
    let mut camera = Camera::new_perspective(
        window.viewport(),
        vec3(4.0, 4.0, 5.0),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    );
    let mut control = OrbitControl::new(*camera.target(), 1.0, 100.0);

    let ambient = AmbientLight::new(&context, 0.4, Color::WHITE);
    let directional = DirectionalLight::new(&context, 2.0, Color::WHITE, &vec3(-1.0, -1.0, -1.0));

    //Instatiate Perceptron, Load training data into the heap, and train the perceptron
    let inputs: [&[f64]; 4] = [
    &[0.0, 0.0],
    &[0.0, 1.0],
    &[1.0, 0.0],
    &[1.0, 1.0],
    ];

    let targets = [1, -1, -1, 1];

    let mut perceptron = Perceptron::new(inputs[0].len(), 0.1);
    perceptron.train(&inputs, &targets, 100);

    for input in &inputs {
        let prediction = perceptron.predict(input);
        println!("Input: {:?} => Prediction: {}", input, prediction);
    }

    //Set up shape meshes
    let mut sphere = CpuMesh::sphere(16);
    sphere.transform(&Mat4::from_translation(Vec3::new(-3.0, 0.0, 0.0))).unwrap();
    let mut sphere_mesh = Gm::new(
        Mesh::new(&context, &sphere),
        PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Color::new(128,128,128,128),
                ..Default::default()
            },
        ),
    );

    let mut input_node_1 = CpuMesh::sphere(16);
    input_node_1.transform(&Mat4::from_translation(Vec3::new(-3.0, 2.0, 0.0))).unwrap();

    let mut input_node_1_mesh = Gm::new(
        Mesh::new(&context, &input_node_1),
        PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Color::new(128,128,128,128),
                ..Default::default()
            },
        ),
    );

    let mut output_node = CpuMesh::sphere(16);
    output_node.transform(&Mat4::from_translation(Vec3::new(1.5, 0.0, 0.0))).unwrap();

    let mut output_node_mesh = Gm::new(
        Mesh::new(&context, &output_node),
        PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Color::new(128,128,128,128),
                ..Default::default()
            },
        ),
    );

    let mut weight_line_1 = CpuMesh::cylinder(8);
    weight_line_1.transform(&Mat4::from_nonuniform_scale(4.0,  0.1, 0.1));
    
    weight_line_1.transform(&Mat4::from_translation(Vec3::new(-3.0, 2.0, 0.0))).unwrap();

    let mut weight_line_1_mesh = Gm::new(
        Mesh::new(&context, &weight_line_1),
        PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Color::new(128,128,128,128),
                ..Default::default()
            },
        ),
    );

    let mut line = CpuMesh::cylinder(8);
    line.transform(&Mat4::from_nonuniform_scale(4.0,  0.1, 0.1));
    line.transform(&Mat4::from_translation(Vec3::new(-3.0, 0.0, 0.0))).unwrap();

    let mut line_mesh = Gm::new(
        Mesh::new(&context, &line),
        PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Color::new(128,128,128,128),
                ..Default::default()
            },
        ),
    );

    //Create GUI context for sidebar
    let mut gui = three_d::GUI::new(&context);
    let mut selected_object: Option<String> = None;

    //Get prediction from perceptron
    for input in &inputs {
        let prediction = perceptron.predict(input);
        let message = format!("Input: {:?} => Prediction: {}", input, prediction);
        console::log_1(&message.into());
    }

    // render loop
    window.render_loop(move |mut frame_input| {
        let mut change = frame_input.first_frame;
        change |= camera.set_viewport(frame_input.viewport);

        //handle mouse click events
        for event in frame_input.events.iter() {
            if let Event::MousePress {
                button, position, ..
            } = event
            {
                if *button == MouseButton::Left {

                    //This has to be in it's own scope for sake of satisfying rust's owernship laws. Will throw a borrow error otherwise
                    //This bit sets all the meshes to grey whenever a new mesh is selected/ user selects empty space
                    {
                        let mut meshes = vec![
                            &mut sphere_mesh,
                            &mut output_node_mesh,
                            &mut line_mesh,
                            &mut input_node_1_mesh,
                            &mut weight_line_1_mesh
                        ];
                        for mesh in &mut meshes {
                            mesh.material.albedo = Color::new(128,128,128,128);
                        }
                    }

                    //if user clicks inside of a mesh, select that mesh
                    if let Some(pick) = pick(&context, &camera, position, &sphere_mesh) {
                        sphere_mesh.material.albedo = Color::RED;
                        selected_object = Some("Sphere".to_string());
                        change = true;
                    } else if let Some(pick) = pick(&context, &camera, position, &output_node_mesh) {
                        output_node_mesh.material.albedo = Color::RED;
                        selected_object = Some("Output Node".to_string());
                        change = true;
                    } else if let Some(pick) = pick(&context, &camera, position, &weight_line_1_mesh) {
                        weight_line_1_mesh.material.albedo = Color::RED;
                        selected_object = Some(perceptron.weights[0].to_string());
                        change = true;
                    } 
                    else if let Some(pick) = pick(&context, &camera, position, &line_mesh) {
                        line_mesh.material.albedo = Color::RED;
                        selected_object = Some(perceptron.weights[1].to_string());
                        change = true;
                    }
                    else if let Some(pick) = pick(&context, &camera, position, &input_node_1_mesh) {
                        input_node_1_mesh.material.albedo = Color::RED;
                        selected_object = Some("Input 1".to_string());
                        change = true;
                    }
                }
            }
        }

        //Render GUI
        let mut panel_width = 0.0;
        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |gui_context| {
                use three_d::egui::*;
                SidePanel::left("side_panel").show(gui_context, |ui| {
                    use three_d::egui::*;
                    ui.heading("Control Panel");
                    if let Some(object) = &selected_object {
                        ui.label(format!("Selected: {}", object));
                    }
                });
                panel_width = gui_context.used_rect().width();
            },
        );

        change |= control.handle_events(&mut camera, &mut frame_input.events);

        // draw three-d objects
        if change {
            frame_input
                .screen()
                .clear(ClearState::color_and_depth(1.0, 1.0, 1.0, 1.0, 1.0))
                .render(
                    &camera,
                    &[&sphere_mesh, &output_node_mesh, &line_mesh, &input_node_1_mesh, &weight_line_1_mesh],
                    &[&ambient, &directional],
                )
                .write(|| gui.render()
            );
        }

        FrameOutput {
            swap_buffers: change,
            ..Default::default()
        }
    });
}
