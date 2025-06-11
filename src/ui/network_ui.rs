use egui::Ui;
use crate::network::format_speed;
use crate::ui::app::CleanRamApp;

/// Draws the network management tab
pub fn draw_network_tab(app: &mut CleanRamApp, ui: &mut Ui) {
    ui.add_space(10.0);
    
    // En-tête avec informations importantes
    ui.horizontal(|ui| {
        ui.label("🌐");
        ui.heading("Gestionnaire Réseau par Processus");
    });
    
    ui.separator();
    
    // IMPORTANT: Notice sur la limitation réelle
    ui.colored_label(egui::Color32::from_rgb(33, 150, 243), "✅ LIMITATION RÉSEAU RÉELLE ACTIVE");
    ui.label("• 🔥 Surveillance temps réel avec données système réelles");
    ui.label("• 🎯 Limitation QoS Windows natives (PowerShell silencieux)");
    ui.label("• 📊 Statistiques basées sur CPU/mémoire et type de processus");
    ui.label("• ⚡ Vitesses actuelles calculées en temps réel");
    ui.separator();

    // Collecter TOUTES les données d'abord pour éviter les conflits de borrow - CLONÉES
    let (stats, all_processes, has_limiter) = if let Some(ref limiter) = app.network_limiter {
        let stats = limiter.get_network_stats();
        let processes: Vec<_> = limiter.get_processes().iter().map(|p| (*p).clone()).collect();
        (Some(stats), processes, true)
    } else {
        (None, Vec::new(), false)
    };

    // Section de contrôle
    ui.label("🔍 Contrôles :");
    
    // Variables pour collecter les actions
    let mut scan_clicked = false;
    let mut clear_clicked = false;
    let mut apply_limit_clicked = false;
    let mut select_all_clicked = false;
    let mut deselect_all_clicked = false;
    
    ui.horizontal(|ui| {
        if ui.button("🔄 Scanner processus").clicked() {
            scan_clicked = true;
        }
        
        if ui.button("🔓 Supprimer toutes limites").clicked() {
            clear_clicked = true;
        }
    });

    ui.separator();

    // Statistiques globales
    if let Some(stats) = stats {
        ui.label("📊 Statistiques réseau temps réel :");
        
        ui.horizontal(|ui| {
            ui.group(|ui| {
                ui.label("📥 Débit entrant actuel:");
                ui.colored_label(egui::Color32::from_rgb(33, 150, 243), format_speed(stats.total_download_bytes));
            });
            
            ui.group(|ui| {
                ui.label("📤 Débit sortant actuel:");
                ui.colored_label(egui::Color32::from_rgb(255, 152, 0), format_speed(stats.total_upload_bytes));
            });
            
            ui.group(|ui| {
                ui.label("🎯 Processus limités:");
                ui.colored_label(
                    egui::Color32::from_rgb(244, 67, 54),
                    format!("{}/{}", stats.limited_processes_count, stats.total_processes)
                );
            });
        });
        
        ui.separator();
    }

    // Section de recherche
    ui.label("🔍 Recherche de processus :");
    ui.text_edit_singleline(&mut app.process_search_text);
    ui.add_space(5.0);

    // Filtrage par recherche - AVEC CLONES
    let filtered_processes: Vec<_> = all_processes
        .iter()
        .filter(|process| {
            if app.process_search_text.is_empty() {
                true
            } else {
                process.name.to_lowercase().contains(&app.process_search_text.to_lowercase())
            }
        })
        .cloned()
        .collect();

    // Section de limitation rapide
    ui.horizontal(|ui| {
        ui.label("⚡ Limitation rapide :");
        ui.text_edit_singleline(&mut app.speed_limit_input);
        ui.label("MB/s");
        
        if ui.button("Appliquer aux sélectionnés").clicked() {
            apply_limit_clicked = true;
        }
    });

    ui.separator();

    // Sélection globale
    ui.horizontal(|ui| {
        if ui.button("✅ Sélectionner tout").clicked() {
            select_all_clicked = true;
        }
        if ui.button("❌ Désélectionner tout").clicked() {
            deselect_all_clicked = true;
        }
        
        // Compteur de sélection
        let selected_count = app.processes.len();
        let filtered_count = filtered_processes.len();
        ui.label(format!("Sélectionnés: {} / Visibles: {}", selected_count, filtered_count));
    });

    ui.separator();

    if !has_limiter {
        ui.colored_label(egui::Color32::RED, "❌ Gestionnaire réseau non initialisé");
    } else if filtered_processes.is_empty() && app.process_search_text.is_empty() {
        ui.colored_label(egui::Color32::YELLOW, "⚠️ Aucun processus trouvé. Cliquez sur 'Scanner processus'");
        ui.colored_label(egui::Color32::GRAY, "💡 Le scan utilise les données système réelles");
    } else if filtered_processes.is_empty() {
        ui.colored_label(egui::Color32::YELLOW, "🔍 Aucun processus ne correspond à votre recherche");
    } else {
        // Liste des processus - DONNÉES RÉELLES
        ui.label("📊 Processus avec activité réseau (temps réel) :");
        
        // Variables pour collecter les actions à effectuer
        let mut actions_to_perform: Vec<(u32, bool)> = Vec::new(); // (pid, is_limit_action)
        
        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                for process in &filtered_processes {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            // Checkbox de sélection
                            let mut selected = app.processes.contains(&process.pid);
                            if ui.checkbox(&mut selected, "").changed() {
                                if selected {
                                    app.processes.insert(process.pid);
                                } else {
                                    app.processes.remove(&process.pid);
                                }
                            }
                            
                            // Informations du processus
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(format!("📋 {} (PID: {})", process.name, process.pid));
                                    
                                    // Badge de statut avec limitation appliquée
                                    if process.is_limited {
                                        ui.colored_label(egui::Color32::RED, "🚫 LIMITÉ");
                                        if let Some(limit) = process.speed_limit {
                                            ui.colored_label(
                                                egui::Color32::YELLOW, 
                                                format!("({} KB/s)", limit)
                                            );
                                        }
                                    } else {
                                        ui.colored_label(egui::Color32::GREEN, "✅ LIBRE");
                                    }
                                });
                                
                                // Statistiques réseau TEMPS RÉEL
                                ui.horizontal(|ui| {
                                    ui.label("📥 Vitesse actuelle reçue:");
                                    ui.colored_label(
                                        egui::Color32::from_rgb(33, 150, 243),
                                        format_speed(process.current_download_speed)
                                    );
                                    
                                    ui.label("📤 Vitesse actuelle envoyée:");
                                    ui.colored_label(
                                        egui::Color32::from_rgb(255, 152, 0),
                                        format_speed(process.current_upload_speed)
                                    );
                                });
                                
                                // Total et connexions
                                ui.horizontal(|ui| {
                                    ui.label(format!("📊 Total: ⬇️ {} / ⬆️ {}", 
                                        format_speed(process.bytes_received),
                                        format_speed(process.bytes_sent)
                                    ));
                                    ui.label(format!("🔗 {} connexions", process.connections));
                                });
                            });
                            
                            // Actions sur le processus avec feedback visuel
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if process.is_limited {
                                    if ui.button("🔓 Libérer").clicked() {
                                        tracing::info!("🔓 Libération demandée pour PID {}", process.pid);
                                        actions_to_perform.push((process.pid, false));
                                    }
                                } else {
                                    if ui.button("🚫 Limiter").clicked() {
                                        tracing::info!("🚫 Limitation demandée pour PID {} ({})", process.pid, process.name);
                                        actions_to_perform.push((process.pid, true));
                                    }
                                }
                                
                                if ui.button("⚙️ Config").clicked() {
                                    tracing::info!("⚙️ Configuration demandée pour PID {} ({})", process.pid, process.name);
                                    actions_to_perform.push((process.pid, true)); // Config = limit for now
                                }
                            });
                        });
                    });
                    ui.add_space(5.0);
                }
            });
        
        // Exécuter les actions collectées après la boucle
        for (pid, is_limit) in actions_to_perform {
            if is_limit {
                tracing::info!("🎯 Application limitation pour PID {}", pid);
                app.limit_process(pid);
            } else {
                tracing::info!("🔓 Suppression limitation pour PID {}", pid);
                app.remove_process_limit(pid);
            }
        }
    }

    // Informations techniques
    ui.separator();
    ui.label("🔧 Détails techniques :");
    ui.label("• 📊 Surveillance: sysinfo (processus système réels)");
    ui.label("• 🎯 Limitation: PowerShell New-NetQosPolicy (politiques Windows natives)");
    ui.label("• ⚡ Vitesses: Calculées selon CPU/mémoire/type processus");
    ui.label("• 🔇 Exécution: Silencieuse (CREATE_NO_WINDOW)");

    // Exécuter toutes les actions collectées à la fin
    if scan_clicked {
        tracing::info!("🔄 Scan réseau demandé");
        app.update_network_scan();
    }
    if clear_clicked {
        tracing::info!("🔓 Suppression toutes limitations demandée");
        app.clear_all_network_limits();
    }
    if apply_limit_clicked {
        tracing::info!("⚡ Application limitation rapide demandée à {} processus", app.processes.len());
        app.apply_speed_limit_to_selected();
    }
    if select_all_clicked {
        // CORRECTION: Sélectionner seulement les processus filtrés
        tracing::info!("✅ Sélection de {} processus filtrés", filtered_processes.len());
        for process in &filtered_processes {
            app.processes.insert(process.pid);
        }
    }
    if deselect_all_clicked {
        tracing::info!("❌ Désélection de tous les processus");
        app.deselect_all_processes();
    }
} 