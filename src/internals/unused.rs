/*
                       #[cfg(cpy_parallel)]
                        {
    let start_parallel = std::time::Instant::now();
                            let mut handles: Vec<std::thread::JoinHandle<()>> = Vec::new();
                            for i in paths_from {
                                let counter_clone = counter.clone();
                                let path_to_clone = path_to.clone();
                                let handle: std::thread::JoinHandle<()> = std::thread::spawn(move || {
                                    cpy_task(vec![i.clone()], path_to_clone, counter_clone);
                                });
                                handles.push(handle);
                            }
                            for handle in handles {
                                handle.join();
                            }
    let duration_parallel = start_parallel.elapsed();
    println!("Copying in parallel finished:{}", duration_parallel.as_secs());
                        }
*/
