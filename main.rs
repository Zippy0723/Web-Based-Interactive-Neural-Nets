// Entry point for non-wasm
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    run().await;
}

use three_d::*;

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

    let mut cube = CpuMesh::cube();
    cube.transform(&Mat4::from_translation(Vec3::new(1.5, 0.0, 0.0))).unwrap();

    let mut cube_mesh = Gm::new(
        Mesh::new(&context, &cube),
        PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Color::new(128,128,128,128),
                ..Default::default()
            },
        ),
    );

    let mut cone = CpuMesh::cone(16);
    cone.transform(&Mat4::from_translation(Vec3::new(1.2, -3.0, 0.0))).unwrap();

    let mut cone_mesh = Gm::new(
        Mesh::new(&context, &cone),
        PhysicalMaterial::new_opaque(
            &context,
            &CpuMaterial {
                albedo: Color::new(128,128,128,128),
                ..Default::default()
            },
        ),
    );

    let mut cylinder = CpuMesh::cylinder(16);
    cylinder.transform(&Mat4::from_translation(Vec3::new(-3.3, -3.0, 0.0))).unwrap();

    let mut cylinder_mesh = Gm::new(
        Mesh::new(&context, &cylinder),
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
                            &mut cube_mesh,
                            &mut cone_mesh,
                            &mut cylinder_mesh,
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
                    } else if let Some(pick) = pick(&context, &camera, position, &cube_mesh) {
                        cube_mesh.material.albedo = Color::RED;
                        selected_object = Some("Cube".to_string());
                        change = true;
                    } else if let Some(pick) = pick(&context, &camera, position, &cone_mesh) {
                        cone_mesh.material.albedo = Color::RED;
                        selected_object = Some("Cone".to_string());
                        change = true;
                    } else if let Some(pick) = pick(&context, &camera, position, &cylinder_mesh) {
                        cylinder_mesh.material.albedo = Color::RED;
                        selected_object = Some("Cylinder".to_string());
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
                    &[&sphere_mesh, &cube_mesh, &cone_mesh, &cylinder_mesh],
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
