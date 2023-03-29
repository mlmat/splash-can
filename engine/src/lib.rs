use ash::{vk, vk_make_version};
use ash::{Instance, Entry, Device};
use ash::version::{EntryV1_0, DeviceV1_0, InstanceV1_0, InstanceV1_1};
use ash::extensions::khr::{Surface, Swapchain};
use std::ptr;
use std::ffi::{CString, CStr};
use std::os::raw::c_void;
use platforms::required_extension_names;
use gui;
use cgci::Draw;

mod platforms;
mod validation;
mod pipeline;

const APPLICATION_VERSION: u32 = vk_make_version!(1, 0, 0);
const ENGINE_VERSION: u32 = vk_make_version!(1, 0, 0);
const API_VERSION: u32 = vk_make_version!(1, 2, 183);

const ENGINE_NAME: &str = "PaintGraphicsEngine";

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct VulkanEngine {
    entry: Entry,
    instance: Instance,
    device: Device,
    device_index: u32,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    surface_loader: Surface,
    surface: vk::SurfaceKHR,
    validation_layers_enabled: bool,
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_messenger: vk::DebugUtilsMessengerEXT,
    swapchain_loader: Swapchain,
    swapchain: vk::SwapchainKHR,
    swapchain_format: vk::Format,
    swapchain_images: Vec<vk::Image>,
    swapchain_extent: vk::Extent2D,
    swapchain_imageviews: Vec<vk::ImageView>,
    swapchain_framebuffers: Vec<vk::Framebuffer>,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    render_pass: vk::RenderPass,
    command_buffers: Vec<vk::CommandBuffer>,
    command_pool: vk::CommandPool,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    current_frame: usize,
}

impl VulkanEngine {
    pub fn new(app_name: &str, validation_layers: bool, window: &gui::MainWindow) -> Self {
        let entry = unsafe { Entry::new() }.unwrap();
        let instance = VulkanEngine::create_instance(app_name, &entry, validation_layers);
        let (debug_utils_loader, debug_messenger) = VulkanEngine::setup_debug_utils(
            &entry,
            &instance,
            validation_layers
        );
        let surface_bundle = VulkanEngine::create_surface(&entry, &instance, &window);
        let device_bundle = VulkanEngine::create_device(
            &instance,
            validation_layers,
            &surface_bundle.surface_loader,
            surface_bundle.surface,
        );
        let swapchain_bundle = VulkanEngine::create_swapchain(
            &device_bundle,
            &instance,
            &surface_bundle,
        );
        let render_pass = pipeline::create_render_pass(
            &device_bundle.logical_device,
            swapchain_bundle.swapchain_format,
        );

        let swapchain_image_views = VulkanEngine::create_image_views(
            &device_bundle.logical_device,
            swapchain_bundle.swapchain_format,
            &swapchain_bundle.swapchain_images,
        );

        let (pipeline_layout, pipeline) = pipeline::create_graphics_pipeline(
            &device_bundle.logical_device,
            swapchain_bundle.swapchain_extent,
            render_pass,
        );

        let framebuffers = VulkanEngine::create_framebuffers(
            &device_bundle.logical_device,
            render_pass,
            &swapchain_image_views,
            swapchain_bundle.swapchain_extent,
        );

        let command_bundle = VulkanEngine::create_command_buffers(
            &device_bundle.logical_device,
            device_bundle.physical_device_index,
            pipeline,
            &framebuffers,
            render_pass,
            swapchain_bundle.swapchain_extent,
        );

        let sync_bundle = VulkanEngine::create_sync_objects(&device_bundle.logical_device);

        Self {
            entry,
            instance,
            device: device_bundle.logical_device,
            graphics_queue: device_bundle.queue,
            present_queue: device_bundle.present_queue,
            device_index: device_bundle.physical_device_index,
            surface_loader: surface_bundle.surface_loader,
            surface: surface_bundle.surface,
            validation_layers_enabled: validation_layers,
            debug_utils_loader,
            debug_messenger,
            swapchain_loader: swapchain_bundle.swapchain_loader,
            swapchain: swapchain_bundle.swapchain,
            swapchain_format: swapchain_bundle.swapchain_format,
            swapchain_images: swapchain_bundle.swapchain_images,
            swapchain_extent: swapchain_bundle.swapchain_extent,
            swapchain_imageviews: swapchain_image_views,
            swapchain_framebuffers: framebuffers,
            pipeline_layout,
            pipeline,
            render_pass,
            command_buffers: command_bundle.command_buffers,
            command_pool: command_bundle.command_pool,
            image_available_semaphores: sync_bundle.image_available_semaphores,
            render_finished_semaphores: sync_bundle.render_finished_semaphores,
            in_flight_fences: sync_bundle.inflight_fences,
            current_frame: 1,
        }
    }

    fn setup_debug_utils(
        entry: &Entry,
        instance: &Instance,
        validation_layers_enabled: bool
    ) -> (ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT) {
        let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, instance);
    
        if !validation_layers_enabled {
            (debug_utils_loader, ash::vk::DebugUtilsMessengerEXT::null())
        } else {
            let messenger_ci = populate_debug_messenger_create_info();
    
            let utils_messenger = unsafe {
                debug_utils_loader
                    .create_debug_utils_messenger(&messenger_ci, None)
                    .expect("Debug Utils Callback")
            };
    
            (debug_utils_loader, utils_messenger)
        }
    }

    fn create_instance(app_name: &str, entry: &Entry, validation_layers_enabled: bool) -> Instance {
        if validation_layers_enabled && !validation::check_validation_layer_support(&entry) {
            panic!("Validation layers requested, but not available!")
        }
        let app_name = CString::new(app_name).unwrap();
        let engine_name = CString::new(ENGINE_NAME).unwrap();

        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(APPLICATION_VERSION)
            .engine_name(&engine_name)
            .engine_version(ENGINE_VERSION)
            .api_version(API_VERSION);

        let enabled_extension_names = required_extension_names();
        let validation_layer_names = validation::get_validation_layers();
        let mut debug_utils_create_info = populate_debug_messenger_create_info();

        let mut create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&enabled_extension_names);
        
        if validation_layers_enabled {
            create_info = create_info
                .push_next(&mut debug_utils_create_info)
                .enabled_layer_names(&validation_layer_names);
        }
    
        let instance: Instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create instance!")
        };
        instance
    }

    fn pick_physical_device(instance: &Instance, surface_loader: &Surface, surface: vk::SurfaceKHR) -> (u32, vk::PhysicalDevice) {
        let devices = unsafe { 
            instance
                .enumerate_physical_devices()
                .expect("Error while enumerating physical devices!") };
        if devices.len() <= 0 {
            panic!("No suitable physical device found!")
        }
        
        let mut integrated_device = None;
        let physical_device_with_index = devices.iter()
            .map(|device|{
                unsafe {
                    (instance.get_physical_device_properties(*device),
                    device)
                }
            })
            .map(|(device_properties, device)| {
                
                unsafe { 
                    instance.get_physical_device_queue_family_properties(*device) 
                }
                    .iter()
                    .enumerate()
                    .filter_map(|(index, ref info)| {
                        let supports_graphic_and_surface = info
                            .queue_flags
                            .contains(vk::QueueFlags::GRAPHICS)
                            && unsafe {
                                    surface_loader.get_physical_device_surface_support(
                                    *device,
                                    index as u32,
                                    surface,
                                )
                            };
                        match supports_graphic_and_surface {
                            true => match device_properties.device_type {
                                vk::PhysicalDeviceType::DISCRETE_GPU => Some((index, *device)),
                                vk::PhysicalDeviceType::INTEGRATED_GPU => {
                                    integrated_device = Some((index, *device));
                                    None
                                },
                                _ => None
                            },
                            _ => None,
                        }
                    })
                    .nth(0)
            })
            .filter_map(|v| v)
            .nth(0);

        match physical_device_with_index {
            Some((index, device)) => (index as u32, device),
            None => match integrated_device {
                Some((index, device)) => (index as u32, device),
                None => panic!("No suitable device found"),
            }
        }
    }

    fn create_device(instance: &Instance, validation_layers: bool, surface_loader: &Surface, surface: vk::SurfaceKHR) 
        -> DeviceBundle {
        
        unsafe {
            let (queue_index, physical_device) = VulkanEngine::pick_physical_device(instance, surface_loader, surface);
            let queue_priorities = [1.0];
            let mut physical_device_features = vk::PhysicalDeviceFeatures2::default();
            instance
                .fp_v1_1()
                .get_physical_device_features2(physical_device, &mut physical_device_features);
            let queue_infos = [vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_index)
                .queue_priorities(&queue_priorities)
                .build()];
    
            let device_extensions = vec![
                Swapchain::name().as_ptr(),
            ];
    
            let mut device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(&queue_infos)
                .enabled_extension_names(&device_extensions)
                .enabled_features(&physical_device_features.features);
    
            let validation_layer_names = validation::get_validation_layers();
            if validation_layers {
                device_create_info = device_create_info
                    .enabled_layer_names(&validation_layer_names)
            }
            let logical_device = instance
                .create_device(physical_device, &device_create_info, None)
                .unwrap();
            let present_queue = logical_device.get_device_queue(queue_index as u32, 0);

            //
            // let graphics_queue =
            //     unsafe { device_bundle.  device.get_device_queue(family_indices.graphics_family.unwrap(), 0) };
            // let present_queue =
            //     unsafe { device.get_device_queue(family_indices.present_family.unwrap(), 0) };

            DeviceBundle {
                physical_device: physical_device,
                physical_device_index: queue_index,
                logical_device: logical_device,
                present_queue: present_queue,
                queue: present_queue,
            }
        }
    }

    fn create_surface(
        entry: &Entry, 
        instance: &Instance, 
        window: &gui::MainWindow) -> SurfaceBundle {

        let surface = unsafe { platforms::create_surface(entry, instance, &window.window) }
            .expect("Failed creating surface!");
        let surface_loader = Surface::new(entry, instance);
        
        SurfaceBundle {
            surface_loader: surface_loader,
            surface: surface,
            width: window.window_width,
            height: window.window_height,
        }
    }

    fn create_swapchain(device_bundle: &DeviceBundle, 
        instance: &Instance, 
        surface_bundle: &SurfaceBundle) -> SwapchainBundle {
        
        unsafe {
            let present_modes = surface_bundle.surface_loader.get_physical_device_surface_present_modes(device_bundle.physical_device, surface_bundle.surface).unwrap();
            let surface_formats = surface_bundle.surface_loader
                .get_physical_device_surface_formats(device_bundle.physical_device, surface_bundle.surface)
                .expect("Failed to query for surface formats.");

            let mut surface_format = surface_formats.first().unwrap().clone();
            for sf in surface_formats.iter() {
                if sf.format == vk::Format::B8G8R8A8_SRGB
                    && sf.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
                        surface_format = sf.clone();
                }
            };
            let surface_capabilities = surface_bundle.surface_loader
                .get_physical_device_surface_capabilities(device_bundle.physical_device, surface_bundle.surface)
                .unwrap();
            let mut desired_image_count = surface_capabilities.min_image_count + 1;
            if surface_capabilities.max_image_count > 0
                    && desired_image_count > surface_capabilities.max_image_count
            {
                desired_image_count = surface_capabilities.max_image_count;
            }
    
            let present_mode = present_modes
                .iter()
                .cloned()
                .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
                .unwrap_or(vk::PresentModeKHR::FIFO);
            
            let swapchain_loader = Swapchain::new(instance, &device_bundle.logical_device);
            let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(surface_bundle.surface)
                .min_image_count(desired_image_count)
                .image_color_space(surface_format.color_space)
                .image_format(surface_format.format)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .present_mode(present_mode)
                .clipped(true)
                .image_array_layers(1);
            let swapchain = swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .unwrap();
            let swapchain_images = swapchain_loader
                .get_swapchain_images(swapchain)
                .expect("Failed to fetch swapchain images.");
            let extent = match surface_capabilities.current_extent.width {
                u32::MAX => vk::Extent2D {
                    width: surface_bundle.width,
                    height: surface_bundle.height,
                },
                _ => surface_capabilities.current_extent,
            };

            SwapchainBundle {
                swapchain_loader: swapchain_loader,
                swapchain: swapchain,
                swapchain_format: surface_format.format,
                swapchain_images: swapchain_images,
                swapchain_extent: extent,
            }
        }
    }

    fn create_image_view(
        device: &Device,
        swapchain_format: vk::Format,
        image: vk::Image,
        view_type: vk::ImageViewType
    ) -> vk::ImageView {
        let create_info = vk::ImageViewCreateInfo::builder()
            .view_type(view_type)
            .format(swapchain_format)
            .subresource_range(
                vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                }
            )
            .components(
                vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                }
            )
            .image(image)
            .build();
        unsafe {
            device
                .create_image_view(&create_info, None)
                .unwrap()
        }
    }

    fn create_image_views(
        device: &Device,
        surface_format: vk::Format,
        swapchain_images: &Vec<vk::Image>,
    ) -> Vec<vk::ImageView>{
        let mut swapchain_image_views = vec![];
        for &image in swapchain_images.iter() {
            let image_view = VulkanEngine::create_image_view(
                device,
                surface_format,
                image,
                vk::ImageViewType::TYPE_2D,
            );
            swapchain_image_views.push(image_view)
        }
        swapchain_image_views
    }

    fn create_framebuffers(
        device: &Device,
        render_pass: vk::RenderPass,
        swapchain_imageviews: &Vec<vk::ImageView>,
        swapchain_extent: vk::Extent2D,
    ) -> Vec<vk::Framebuffer> {
        let mut swapchain_framebuffers = vec![];

        for &imageview in swapchain_imageviews.iter() {
            let attachments = &[imageview];
            let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(attachments)
                .width(swapchain_extent.width)
                .height(swapchain_extent.height);

            let framebuffer = unsafe {
                device.create_framebuffer(&framebuffer_create_info, None)
                    .expect("Failed to create framebuffer!")
            };
            swapchain_framebuffers.push(framebuffer);
        }

        swapchain_framebuffers
    }

    fn create_command_pool(device: &Device, device_index: u32) -> vk::CommandPool {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(device_index)
            .flags(vk::CommandPoolCreateFlags::empty());

        unsafe {
            device
                .create_command_pool(&command_pool_create_info, None)
                .unwrap()
        }
    }

    fn create_command_buffers(
        device: &Device,
        device_index: u32,
        pipeline: vk::Pipeline,
        swapchain_framebuffers: &Vec<vk::Framebuffer>,
        render_pass: vk::RenderPass,
        swapchain_extent: vk::Extent2D,
    ) -> CommandBundle {
        let command_pool = VulkanEngine::create_command_pool(device, device_index);
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .command_buffer_count(swapchain_framebuffers.len() as u32);
        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to create command buffer!")
        };

        for (i, &cb) in command_buffers.iter().enumerate() {
            let cb_begin_info = vk::CommandBufferBeginInfo::builder();
            unsafe {
                device.begin_command_buffer(cb, &cb_begin_info)
                    .expect("Failed to start command buffer!");
            }

            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(render_pass)
                .framebuffer(swapchain_framebuffers[i])
                .render_area(
                    vk::Rect2D::builder()
                        .extent(swapchain_extent)
                        .build()
                )
                .clear_values(
                    &[
                        vk::ClearValue{
                            color: vk::ClearColorValue{
                                float32: [0.0, 0.0, 0.0, 1.0],
                            }
                        }
                    ]
                );
            unsafe {
                device.cmd_begin_render_pass(
                    cb,
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );
                device.cmd_bind_pipeline(
                    cb,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline,
                );
                device.cmd_draw(cb, 4, 1, 0, 0);
                device.cmd_end_render_pass(cb);
                device.end_command_buffer(cb)
                    .expect("Failed to end command buffer!");
            }
        }
        CommandBundle {
            command_buffers,
            command_pool,
        }
    }

    fn create_sync_objects(device: &Device) -> SyncBundle {
        let semaphore_create_info = vk::SemaphoreCreateInfo::builder().build();
        let fence_create_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();

        let mut sync_bundle = SyncBundle {
            image_available_semaphores: vec![],
            render_finished_semaphores: vec![],
            inflight_fences: vec![],
        };

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                let image_available_semaphore = device.create_semaphore(
                    &semaphore_create_info,
                    None,
                ).expect("Failed to create semaphore for image availability");

                let render_finished_semaphore = device.create_semaphore(
                    &semaphore_create_info,
                    None,
                ).expect("Failed to create semaphore for finished rendering");

                let inflight_fence = device.create_fence(
                    &fence_create_info,
                    None,
                ).expect("Failed to create fence for inflight images");

                sync_bundle.image_available_semaphores.push(image_available_semaphore);
                sync_bundle.render_finished_semaphores.push(render_finished_semaphore);
                sync_bundle.inflight_fences.push(inflight_fence);
            }
        }

        sync_bundle
    }
}

impl Draw for VulkanEngine {
    fn draw_frame(&mut self) {
        let wait_fences = [self.in_flight_fences[self.current_frame]];

        let (image_index, _is_sub_optimal) = unsafe {
            self.device
                .wait_for_fences(&wait_fences, true, std::u64::MAX)
                .expect("Failed to wait for Fence!");

            self.swapchain_loader
                .acquire_next_image(
                    self.swapchain,
                    std::u64::MAX,
                    self.image_available_semaphores[self.current_frame],
                    vk::Fence::null(),
                )
                .expect("Failed to acquire next image.")
        };

        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];

        let submit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &self.command_buffers[image_index as usize],
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
        }];

        unsafe {
            self.device
                .reset_fences(&wait_fences)
                .expect("Failed to reset Fence!");

            self.device
                .queue_submit(
                    self.graphics_queue,
                    &submit_infos,
                    self.in_flight_fences[self.current_frame],
                )
                .expect("Failed to execute queue submit.");
        }

        let swapchains = [self.swapchain];

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_index,
            p_results: ptr::null_mut(),
        };

        unsafe {
            self.swapchain_loader
                .queue_present(self.present_queue, &present_info)
                .expect("Failed to execute queue present.");
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }
}

impl Drop for VulkanEngine {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();

            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.device
                    .destroy_semaphore(self.image_available_semaphores[i], None);
                self.device
                    .destroy_semaphore(self.render_finished_semaphores[i], None);
                self.device.destroy_fence(self.in_flight_fences[i], None);
            }

            self.device.destroy_command_pool(self.command_pool, None);

            for &framebuffer in self.swapchain_framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }

            self.device.destroy_pipeline(self.pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
            for &imageview in self.swapchain_imageviews.iter() {
                self.device.destroy_image_view(imageview, None);
            }
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);

            if self.validation_layers_enabled {
                self.debug_utils_loader
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
            }
            self.instance.destroy_instance(None);
        }
    }
}

struct DeviceBundle {
    pub physical_device: vk::PhysicalDevice,
    pub physical_device_index: u32,
    pub logical_device: Device,
    pub present_queue: vk::Queue,
    pub queue: vk::Queue,
}

struct SurfaceBundle {
    surface_loader: Surface,
    surface: vk::SurfaceKHR,
    width: u32,
    height: u32,
}

struct SwapchainBundle {
    swapchain_loader: Swapchain, 
    swapchain: vk::SwapchainKHR,
    swapchain_format: vk::Format,
    swapchain_images: Vec<vk::Image>,
    swapchain_extent: vk::Extent2D,
}

struct SyncBundle {
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    inflight_fences: Vec<vk::Fence>
}

struct CommandBundle {
    command_buffers: Vec<vk::CommandBuffer>,
    command_pool: vk::CommandPool,
}

#[derive(Clone)]
struct ImageBundle {

}

fn populate_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
    vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next: ptr::null(),
        flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
            // vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
            // vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        pfn_user_callback: Some(vulkan_debug_utils_callback),
        p_user_data: ptr::null_mut(),
    }
}

/// the callback function used in Debug Utils.
unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let severity = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]",
        _ => "[Unknown]",
    };
    let types = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]",
    };
    let message = CStr::from_ptr((*p_callback_data).p_message);
    println!("[Debug]{}{}{:?}", severity, types, message);

    vk::FALSE
}
