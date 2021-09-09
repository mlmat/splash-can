use ash::Device;
use ash::version::DeviceV1_0;
use ash::vk;

pub fn create_graphics_pipeline(
    device: &Device, 
    swapchain_extent: vk::Extent2D,
    render_pass: vk::RenderPass) -> (ash::vk::PipelineLayout, ash::vk::Pipeline) {
    let vert_module_create_info = vk::ShaderModuleCreateInfo::builder();
    let frag_module_create_info = vk::ShaderModuleCreateInfo::builder();
    let (vert_module, frag_module) = unsafe {
        (device.create_shader_module(&vert_module_create_info, None).unwrap(),
        device.create_shader_module(&frag_module_create_info, None).unwrap())
    };
    let shader_stage_create_infos = [
        vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_module)
            .build(),
        vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_module)
            .build()
    ];
    //let vertex_input_description = vk::VertexInputAttributeDescription::builder().build();
    let vertex_input_create_info = vk::PipelineVertexInputStateCreateInfo::builder();
    //    .vertex_attribute_descriptions(&[vertex_input_description]);
    let vertex_input_assembly_create_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .primitive_restart_enable(false)
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
    let viewports = [
        vk::Viewport::builder()
            .x(0 as f32)
            .y(0 as f32)
            .min_depth(0.0)
            .max_depth(1.0)
            .width(swapchain_extent.width as f32)
            .height(swapchain_extent.height as f32)
            .build()
    ];
    let viewport_scissors = [
        vk::Rect2D::builder()
            .extent(swapchain_extent)
            .offset(vk::Offset2D { x: 0, y: 0 })
            .build()
    ];
    let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::builder()
        .scissors(&viewport_scissors)
        .viewports(&viewports);
    let rasterization_state_create_info = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::CLOCKWISE)
        .depth_clamp_enable(false);
    let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo::builder();
    let color_blend_attachment_states = [
        vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false)
            .color_write_mask(vk::ColorComponentFlags::all())
            .build()
    ];
    let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(&color_blend_attachment_states);
    let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder()
        .flags(vk::PipelineDynamicStateCreateFlags::empty())
        .dynamic_states(&dynamic_state);
    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder();
    let pipeline_layout = unsafe {
        device.create_pipeline_layout(&pipeline_layout_create_info, None)
            .unwrap()
    };

    let graphic_pipeline_create_infos = [
        vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stage_create_infos)
            .vertex_input_state(&vertex_input_create_info)
            .input_assembly_state(&vertex_input_assembly_create_info)
            .viewport_state(&viewport_state_create_info)
            .rasterization_state(&rasterization_state_create_info)
            .multisample_state(&multisample_state_create_info)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state_info)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .build()
    ];

    let graphic_pipeline = unsafe {
        device
            .create_graphics_pipelines(vk::PipelineCache::null(), &graphic_pipeline_create_infos, None)
            .expect("Failed to create Graphics Pipeline!.")
    };

    unsafe {
        device.destroy_shader_module(vert_module, None);
        device.destroy_shader_module(frag_module, None);
    }

    (pipeline_layout, graphic_pipeline[0])
}

pub fn create_render_pass(device: &Device, surface_format: vk::Format) -> vk::RenderPass {
    let color_attachments = [
        vk::AttachmentDescription::builder()
            .format(surface_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build()
    ];
    let color_attachment_refs = [
        vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build()
    ];
    let subpasses = [
        vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_refs)
            .build()
    ];
    let renderpass_create_info = vk::RenderPassCreateInfo::builder()
        .attachments(&color_attachments)
        .subpasses(&subpasses);
    unsafe {
        device.create_render_pass(&renderpass_create_info, None)
            .expect("Failed to create render pass!")
    }
}