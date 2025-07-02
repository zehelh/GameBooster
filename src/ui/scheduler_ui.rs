// UI for the internal task scheduler
use crate::ui::app::CleanRamApp;
use crate::scheduler::{ScheduledTask, TaskType, ScheduleRule, TaskScheduler};
use crate::disk::DiskCleaningOptions;
use eframe::egui;
use chrono::{NaiveTime, Weekday, Timelike};

pub fn draw_scheduler_tab(app: &mut CleanRamApp, ui: &mut egui::Ui) {
    ui.heading("⏰ Task Scheduler");
    ui.separator();

    // Scheduler status
    ui.horizontal(|ui| {
        let status_text = if app.scheduler_running {
            "🟢 Scheduler Running"
        } else {
            "🔴 Scheduler Stopped"
        };
        ui.label(status_text);

        if ui.button(if app.scheduler_running { "Stop" } else { "Start" }).clicked() {
            app.scheduler_running = !app.scheduler_running;
        }
    });

    ui.separator();

    // Task list
    ui.horizontal(|ui| {
        ui.heading("📋 Scheduled Tasks");
        if ui.button("➕ Add Task").clicked() {
            app.show_add_task_dialog = true;
        }
    });

    // Display existing tasks
    draw_task_list(app, ui);

    ui.separator();

    // Quick task templates
    ui.heading("🚀 Quick Templates");
    ui.horizontal(|ui| {
        if ui.button("💾 RAM Monitor (85%)").clicked() {
            let task = TaskScheduler::create_default_ram_cleanup_task();
            app.add_scheduled_task(task);
        }

        if ui.button("💽 Daily Disk Clean").clicked() {
            let task = TaskScheduler::create_default_disk_cleanup_task();
            app.add_scheduled_task(task);
        }

        if ui.button("🛡️ Weekly Defender Off").clicked() {
            let task = TaskScheduler::create_default_defender_disable_task();
            app.add_scheduled_task(task);
        }
    });

    // Add task dialog
    if app.show_add_task_dialog {
        draw_add_task_dialog(app, ui);
    }

    // Edit task dialog
    if let Some(task_id) = &app.editing_task_id.clone() {
        if let Some(task) = app.scheduled_tasks.get(task_id).cloned() {
            // Initialize edit dialog fields with task values on first open
            if app.edit_task_name.is_empty() {
                populate_edit_dialog_from_task(app, &task);
            }
            draw_edit_task_dialog(app, ui, task);
        }
    }
}

fn draw_task_list(app: &mut CleanRamApp, ui: &mut egui::Ui) {
    if app.scheduled_tasks.is_empty() {
        ui.label("No scheduled tasks. Use quick templates or add a custom task.");
        return;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (task_id, task) in &app.scheduled_tasks.clone() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    // Enable/disable checkbox
                    let mut enabled = task.enabled;
                    if ui.checkbox(&mut enabled, "").changed() {
                        if let Some(task) = app.scheduled_tasks.get_mut(task_id) {
                            task.enabled = enabled;
                        }
                    }

                    // Task info
                    ui.vertical(|ui| {
                        ui.strong(&task.name);
                        ui.label(&task.description);
                        
                        // Schedule info
                        let schedule_text = format_schedule_rule(&task.schedule);
                        ui.label(format!("📅 {}", schedule_text));

                        // Last run info
                        if let Some(last_run) = &task.last_run {
                            ui.label(format!("🕐 Last run: {}", last_run.format("%Y-%m-%d %H:%M")));
                        }

                        // Statistics
                        ui.horizontal(|ui| {
                            ui.label(format!("✅ {}/{}", task.success_count, task.run_count));
                            if let Some(error) = &task.last_error {
                                ui.colored_label(egui::Color32::RED, format!("❌ {}", error));
                            }
                        });
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑️").clicked() {
                            app.task_to_delete = Some(task_id.clone());
                        }
                        if ui.button("✏️").clicked() {
                            app.editing_task_id = Some(task_id.clone());
                        }
                    });
                });
            });
        }
    });

    // Handle task deletion
    if let Some(task_id) = &app.task_to_delete.clone() {
        app.remove_scheduled_task(task_id);
        app.task_to_delete = None;
    }
}

fn draw_add_task_dialog(app: &mut CleanRamApp, ui: &mut egui::Ui) {
    egui::Window::new("Add New Task")
        .default_width(400.0)
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Task Name:");
                ui.text_edit_singleline(&mut app.new_task_name);
            });

            ui.horizontal(|ui| {
                ui.label("Description:");
                ui.text_edit_singleline(&mut app.new_task_description);
            });

            ui.separator();

            // Task type selection
            ui.label("Task Type:");
            ui.horizontal(|ui| {
                ui.radio_value(&mut app.new_task_type, 0, "💾 Clean RAM");
                ui.radio_value(&mut app.new_task_type, 1, "💽 Clean Disk");
                ui.radio_value(&mut app.new_task_type, 2, "🛡️ Defender Toggle");
            });

            // Task type specific options
            match app.new_task_type {
                0 => {
                    ui.horizontal(|ui| {
                        ui.label("RAM Threshold (%):");
                        ui.add(egui::Slider::new(&mut app.new_task_ram_threshold, 50..=95));
                    });
                }
                1 => {
                    ui.horizontal(|ui| {
                        ui.label("Size Threshold (MB):");
                        ui.add(egui::Slider::new(&mut app.new_task_disk_threshold, 50..=2000));
                    });
                    
                    ui.separator();
                    ui.label("Disk Cleaning Options:");
                    ui.indent("disk_options", |ui| {
                        ui.checkbox(&mut app.disk_options.clean_temp_files, "🗂️ Temporary files");
                        ui.checkbox(&mut app.disk_options.clean_browser_cache, "🌐 Browser cache");
                        ui.checkbox(&mut app.disk_options.clean_thumbnails, "🖼️ Thumbnails cache");
                        ui.checkbox(&mut app.disk_options.clean_recycle_bin, "🗑️ Recycle bin");
                        ui.checkbox(&mut app.disk_options.clean_system_cache, "⚙️ System cache (experimental)");
                    });
                }
                2 => {
                    ui.horizontal(|ui| {
                        ui.label("Action:");
                        ui.radio_value(&mut app.new_task_defender_action, true, "Enable");
                        ui.radio_value(&mut app.new_task_defender_action, false, "Disable");
                    });
                }
                _ => {}
            }

            ui.separator();

            // Schedule type selection
            ui.label("Schedule:");
            ui.horizontal(|ui| {
                ui.radio_value(&mut app.new_task_schedule_type, 0, "On Startup");
                ui.radio_value(&mut app.new_task_schedule_type, 1, "Interval");
                ui.radio_value(&mut app.new_task_schedule_type, 2, "Daily");
                ui.radio_value(&mut app.new_task_schedule_type, 3, "Weekly");
                if app.new_task_type < 2 {
                    ui.radio_value(&mut app.new_task_schedule_type, 4, "On Condition");
                }
            });

            // Schedule specific options
            match app.new_task_schedule_type {
                1 => {
                    ui.horizontal(|ui| {
                        ui.label("Every");
                        ui.add(egui::Slider::new(&mut app.new_task_interval_minutes, 5..=1440));
                        ui.label("minutes");
                    });
                }
                2 => {
                    ui.horizontal(|ui| {
                        ui.label("Time:");
                        ui.add(egui::Slider::new(&mut app.new_task_daily_hour, 0..=23));
                        ui.label(":");
                        ui.add(egui::Slider::new(&mut app.new_task_daily_minute, 0..=59));
                    });
                }
                3 => {
                    ui.horizontal(|ui| {
                        ui.label("Day:");
                        egui::ComboBox::from_label("")
                            .selected_text(format!("{:?}", get_weekday_from_index(app.new_task_weekly_day)))
                            .show_ui(ui, |ui| {
                                for (i, day) in ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"].iter().enumerate() {
                                    ui.selectable_value(&mut app.new_task_weekly_day, i, *day);
                                }
                            });
                    });
                    ui.horizontal(|ui| {
                        ui.label("Time:");
                        ui.add(egui::Slider::new(&mut app.new_task_weekly_hour, 0..=23));
                        ui.label(":");
                        ui.add(egui::Slider::new(&mut app.new_task_weekly_minute, 0..=59));
                    });
                }
                _ => {}
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Create").clicked() {
                    let task = create_task_from_dialog(app);
                    app.add_scheduled_task(task);
                    reset_new_task_dialog(app);
                    app.show_add_task_dialog = false;
                }
                if ui.button("Cancel").clicked() {
                    reset_new_task_dialog(app);
                    app.show_add_task_dialog = false;
                }
            });
        });
}

fn draw_edit_task_dialog(app: &mut CleanRamApp, ui: &mut egui::Ui, task: ScheduledTask) {
    egui::Window::new("Edit Task")
        .default_width(500.0)
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Task Name:");
                ui.text_edit_singleline(&mut app.edit_task_name);
            });

            ui.horizontal(|ui| {
                ui.label("Description:");
                ui.text_edit_singleline(&mut app.edit_task_description);
            });

            ui.separator();

            // Task type selection
            ui.label("Task Type:");
            ui.horizontal(|ui| {
                ui.radio_value(&mut app.edit_task_type, 0, "💾 Clean RAM");
                ui.radio_value(&mut app.edit_task_type, 1, "💽 Clean Disk");
                ui.radio_value(&mut app.edit_task_type, 2, "🛡️ Defender Toggle");
            });

            // Task type specific options
            match app.edit_task_type {
                0 => {
                    ui.horizontal(|ui| {
                        ui.label("RAM Threshold (%):");
                        ui.add(egui::Slider::new(&mut app.edit_task_ram_threshold, 50..=95));
                    });
                }
                1 => {
                    ui.horizontal(|ui| {
                        ui.label("Size Threshold (MB):");
                        ui.add(egui::Slider::new(&mut app.edit_task_disk_threshold, 50..=2000));
                    });
                    
                    ui.separator();
                    ui.label("Disk Cleaning Options:");
                    ui.indent("edit_disk_options", |ui| {
                        ui.checkbox(&mut app.edit_disk_options.clean_temp_files, "🗂️ Temporary files");
                        ui.checkbox(&mut app.edit_disk_options.clean_browser_cache, "🌐 Browser cache");
                        ui.checkbox(&mut app.edit_disk_options.clean_thumbnails, "🖼️ Thumbnails cache");
                        ui.checkbox(&mut app.edit_disk_options.clean_recycle_bin, "🗑️ Recycle bin");
                        ui.checkbox(&mut app.edit_disk_options.clean_system_cache, "⚙️ System cache (experimental)");
                    });
                }
                2 => {
                    ui.horizontal(|ui| {
                        ui.label("Action:");
                        ui.radio_value(&mut app.edit_task_defender_action, true, "Enable");
                        ui.radio_value(&mut app.edit_task_defender_action, false, "Disable");
                    });
                }
                _ => {}
            }

            ui.separator();

            // Schedule type selection
            ui.label("Schedule:");
            ui.horizontal(|ui| {
                ui.radio_value(&mut app.edit_task_schedule_type, 0, "On Startup");
                ui.radio_value(&mut app.edit_task_schedule_type, 1, "Interval");
                ui.radio_value(&mut app.edit_task_schedule_type, 2, "Daily");
                ui.radio_value(&mut app.edit_task_schedule_type, 3, "Weekly");
                if app.edit_task_type < 2 {
                    ui.radio_value(&mut app.edit_task_schedule_type, 4, "On Condition");
                }
            });

            // Schedule specific options
            match app.edit_task_schedule_type {
                1 => {
                    ui.horizontal(|ui| {
                        ui.label("Every");
                        ui.add(egui::Slider::new(&mut app.edit_task_interval_minutes, 5..=1440));
                        ui.label("minutes");
                    });
                }
                2 => {
                    ui.horizontal(|ui| {
                        ui.label("Time:");
                        ui.add(egui::Slider::new(&mut app.edit_task_daily_hour, 0..=23));
                        ui.label(":");
                        ui.add(egui::Slider::new(&mut app.edit_task_daily_minute, 0..=59));
                    });
                }
                3 => {
                    ui.horizontal(|ui| {
                        ui.label("Day:");
                        egui::ComboBox::from_label("")
                            .selected_text(format!("{:?}", get_weekday_from_index(app.edit_task_weekly_day)))
                            .show_ui(ui, |ui| {
                                for (i, day) in ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"].iter().enumerate() {
                                    ui.selectable_value(&mut app.edit_task_weekly_day, i, *day);
                                }
                            });
                    });
                    ui.horizontal(|ui| {
                        ui.label("Time:");
                        ui.add(egui::Slider::new(&mut app.edit_task_weekly_hour, 0..=23));
                        ui.label(":");
                        ui.add(egui::Slider::new(&mut app.edit_task_weekly_minute, 0..=59));
                    });
                }
                _ => {}
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Save Changes").clicked() {
                    let updated_task = create_updated_task_from_edit_dialog(app, &task);
                    app.update_scheduled_task(updated_task);
                    reset_edit_task_dialog(app);
                    app.editing_task_id = None;
                }
                if ui.button("Cancel").clicked() {
                    reset_edit_task_dialog(app);
                    app.editing_task_id = None;
                }
            });
        });
}

fn create_task_from_dialog(app: &CleanRamApp) -> ScheduledTask {
    let task_type = match app.new_task_type {
        0 => TaskType::CleanRam { threshold_percentage: app.new_task_ram_threshold },
        1 => TaskType::CleanDisk { 
            size_threshold_mb: app.new_task_disk_threshold as u64,
            options: app.disk_options.clone(),
        },
        2 => TaskType::DefenderToggle { enable: app.new_task_defender_action },
        _ => TaskType::CleanRam { threshold_percentage: 85 },
    };

    let schedule = match app.new_task_schedule_type {
        0 => ScheduleRule::OnStartup,
        1 => ScheduleRule::Interval { minutes: app.new_task_interval_minutes },
        2 => ScheduleRule::Daily { 
            time: NaiveTime::from_hms_opt(app.new_task_daily_hour, app.new_task_daily_minute, 0).unwrap_or_default()
        },
        3 => ScheduleRule::Weekly { 
            weekday: get_weekday_from_index(app.new_task_weekly_day),
            time: NaiveTime::from_hms_opt(app.new_task_weekly_hour, app.new_task_weekly_minute, 0).unwrap_or_default()
        },
        4 => ScheduleRule::OnCondition,
        _ => ScheduleRule::OnStartup,
    };

    let id = format!("task_{}", chrono::Utc::now().timestamp());
    
    ScheduledTask::new(
        id,
        app.new_task_name.clone(),
        app.new_task_description.clone(),
        task_type,
        schedule,
    )
}

fn reset_new_task_dialog(app: &mut CleanRamApp) {
    app.new_task_name.clear();
    app.new_task_description.clear();
    app.new_task_type = 0;
    app.new_task_ram_threshold = 85;
    app.new_task_disk_threshold = 100;
    app.new_task_defender_action = false;
    app.new_task_schedule_type = 0;
    app.new_task_interval_minutes = 60;
    app.new_task_daily_hour = 2;
    app.new_task_daily_minute = 0;
    app.new_task_weekly_day = 0;
    app.new_task_weekly_hour = 9;
    app.new_task_weekly_minute = 0;
}

fn reset_edit_task_dialog(app: &mut CleanRamApp) {
    app.edit_task_name.clear();
    app.edit_task_description.clear();
    app.edit_task_type = 0;
    app.edit_task_ram_threshold = 85;
    app.edit_task_disk_threshold = 100;
    app.edit_task_defender_action = false;
    app.edit_task_schedule_type = 0;
    app.edit_task_interval_minutes = 60;
    app.edit_task_daily_hour = 2;
    app.edit_task_daily_minute = 0;
    app.edit_task_weekly_day = 0;
    app.edit_task_weekly_hour = 9;
    app.edit_task_weekly_minute = 0;
    app.edit_disk_options = DiskCleaningOptions::default();
}

fn format_schedule_rule(rule: &ScheduleRule) -> String {
    match rule {
        ScheduleRule::OnStartup => "On application startup".to_string(),
        ScheduleRule::Interval { minutes } => format!("Every {} minutes", minutes),
        ScheduleRule::Daily { time } => format!("Daily at {}", time.format("%H:%M")),
        ScheduleRule::Weekly { weekday, time } => format!("{:?} at {}", weekday, time.format("%H:%M")),
        ScheduleRule::OnCondition => "When condition is met".to_string(),
    }
}

fn get_weekday_from_index(index: usize) -> Weekday {
    match index {
        0 => Weekday::Mon,
        1 => Weekday::Tue,
        2 => Weekday::Wed,
        3 => Weekday::Thu,
        4 => Weekday::Fri,
        5 => Weekday::Sat,
        6 => Weekday::Sun,
        _ => Weekday::Mon,
    }
}

fn get_index_from_weekday(weekday: Weekday) -> usize {
    match weekday {
        Weekday::Mon => 0,
        Weekday::Tue => 1,
        Weekday::Wed => 2,
        Weekday::Thu => 3,
        Weekday::Fri => 4,
        Weekday::Sat => 5,
        Weekday::Sun => 6,
    }
}

fn populate_edit_dialog_from_task(app: &mut CleanRamApp, task: &ScheduledTask) {
    app.edit_task_name = task.name.clone();
    app.edit_task_description = task.description.clone();
    
    // Set task type and specific parameters
    match &task.task_type {
        TaskType::CleanRam { threshold_percentage } => {
            app.edit_task_type = 0;
            app.edit_task_ram_threshold = *threshold_percentage;
        }
        TaskType::CleanDisk { size_threshold_mb, options } => {
            app.edit_task_type = 1;
            app.edit_task_disk_threshold = *size_threshold_mb as u32;
            app.edit_disk_options = options.clone();
        }
        TaskType::DefenderToggle { enable } => {
            app.edit_task_type = 2;
            app.edit_task_defender_action = *enable;
        }
    }
    
    // Set schedule type and specific parameters
    match &task.schedule {
        ScheduleRule::OnStartup => {
            app.edit_task_schedule_type = 0;
        }
        ScheduleRule::Interval { minutes } => {
            app.edit_task_schedule_type = 1;
            app.edit_task_interval_minutes = *minutes;
        }
        ScheduleRule::Daily { time } => {
            app.edit_task_schedule_type = 2;
            app.edit_task_daily_hour = time.hour();
            app.edit_task_daily_minute = time.minute();
        }
        ScheduleRule::Weekly { weekday, time } => {
            app.edit_task_schedule_type = 3;
            app.edit_task_weekly_day = get_index_from_weekday(*weekday);
            app.edit_task_weekly_hour = time.hour();
            app.edit_task_weekly_minute = time.minute();
        }
        ScheduleRule::OnCondition => {
            app.edit_task_schedule_type = 4;
        }
    }
}

fn create_updated_task_from_edit_dialog(app: &CleanRamApp, original_task: &ScheduledTask) -> ScheduledTask {
    let task_type = match app.edit_task_type {
        0 => TaskType::CleanRam { threshold_percentage: app.edit_task_ram_threshold },
        1 => TaskType::CleanDisk { 
            size_threshold_mb: app.edit_task_disk_threshold as u64,
            options: app.edit_disk_options.clone(),
        },
        2 => TaskType::DefenderToggle { enable: app.edit_task_defender_action },
        _ => TaskType::CleanRam { threshold_percentage: 85 },
    };

    let schedule = match app.edit_task_schedule_type {
        0 => ScheduleRule::OnStartup,
        1 => ScheduleRule::Interval { minutes: app.edit_task_interval_minutes },
        2 => ScheduleRule::Daily { 
            time: NaiveTime::from_hms_opt(app.edit_task_daily_hour, app.edit_task_daily_minute, 0).unwrap_or_default()
        },
        3 => ScheduleRule::Weekly { 
            weekday: get_weekday_from_index(app.edit_task_weekly_day),
            time: NaiveTime::from_hms_opt(app.edit_task_weekly_hour, app.edit_task_weekly_minute, 0).unwrap_or_default()
        },
        4 => ScheduleRule::OnCondition,
        _ => ScheduleRule::OnStartup,
    };

    ScheduledTask {
        id: original_task.id.clone(), // Keep the same ID
        name: app.edit_task_name.clone(),
        description: app.edit_task_description.clone(),
        task_type,
        schedule,
        enabled: original_task.enabled, // Keep current enabled state
        last_run: original_task.last_run, // Keep execution history
        run_count: original_task.run_count,
        success_count: original_task.success_count,
        last_error: original_task.last_error.clone(),
        next_run: None, // Will be recalculated
    }
}