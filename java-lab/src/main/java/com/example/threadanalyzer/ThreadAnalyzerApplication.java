package com.example.threadanalyzer;

import static spark.Spark.*;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class ThreadAnalyzerApplication {

    private static final Logger logger = LoggerFactory.getLogger(ThreadAnalyzerApplication.class);
    private static final Object blockMonitor = new Object(); // Monitor for the /block endpoint
    private static final Object deadlockLock1 = new Object();
    private static final Object deadlockLock2 = new Object();
    private static final java.util.concurrent.CountDownLatch blockerLatch = new java.util.concurrent.CountDownLatch(1);
    private static final java.util.List<Thread> managedThreads = new java.util.ArrayList<>();

    // New method for blocking
    public static void simulateBlock() {
        logger.info("Thread {} attempting to acquire blockMonitor...", Thread.currentThread().getName());
        synchronized (blockMonitor) {
            logger.info("Thread {} acquired blockMonitor, entering indefinite wait.", Thread.currentThread().getName());
            try {
                // Hold the lock indefinitely
                blockMonitor.wait(); 
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
                logger.warn("Thread {} interrupted while holding blockMonitor.", Thread.currentThread().getName());
            }
            logger.info("Thread {} released blockMonitor.", Thread.currentThread().getName());
        }
    }

    // New method for acquiring deadlockLock1
    public static void acquireDeadlockLock1() {
        logger.info("DeadlockThread-1: Attempting to acquire deadlockLock1...");
        synchronized (deadlockLock1) {
            logger.info("DeadlockThread-1: Acquired deadlockLock1. Holding...");
            try {
                Thread.sleep(100);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            }
            logger.info("DeadlockThread-1: Attempting to acquire deadlockLock2...");
            synchronized (deadlockLock2) {
                logger.info("DeadlockThread-1: Acquired deadlockLock2. Holding both locks.");
            }
        }
        logger.info("DeadlockThread-1: Released both locks.");
    }

    // New method for acquiring deadlockLock2
    public static void acquireDeadlockLock2() {
        logger.info("DeadlockThread-2: Attempting to acquire deadlockLock2...");
        synchronized (deadlockLock2) {
            logger.info("DeadlockThread-2: Acquired deadlockLock2. Holding...");
            try {
                Thread.sleep(100);
            } catch (InterruptedException e) {
                Thread.currentThread().interrupt();
            }
            logger.info("DeadlockThread-2: Attempting to acquire deadlockLock1...");
            synchronized (deadlockLock1) {
                logger.info("DeadlockThread-2: Acquired deadlockLock1. Holding both locks.");
            }
        }
        logger.info("DeadlockThread-2: Released both locks.");
    }


    public static void main(String[] args) {
        port(8080);

        get("/", (req, res) -> {
            return "Hello World from Spark Java!";
        });

        get("/block", (req, res) -> {
            logger.info("Received /block request. Initiating blocking scenario.");
            Thread blockerThread = new Thread(() -> {
                synchronized (blockMonitor) {
                    logger.info("BlockerThread acquired blockMonitor. Will hold indefinitely.");
                    // Hold the lock indefinitely without releasing it
                    try {
                        blockerLatch.await(); // This thread will wait indefinitely, holding the lock
                    } catch (InterruptedException e) {
                        Thread.currentThread().interrupt();
                        logger.warn("BlockerThread interrupted while holding blockMonitor.");
                    }
                }
            }, "BlockerThread");
            managedThreads.add(blockerThread);

            // Thread that attempts to acquire the same lock and gets BLOCKED
            Thread blockedThread = new Thread(() -> {
                logger.info("BlockedThread attempting to acquire blockMonitor...");
                synchronized (blockMonitor) { // This thread will block here
                    logger.info("BlockedThread acquired blockMonitor (should not happen if BlockerThread is active).");
                }
            }, "BlockedThread");
            managedThreads.add(blockedThread);

            blockerThread.start();
            blockedThread.start();

            return "Blocking scenario initiated!";
        });

        get("/deadlock", (req, res) -> {
            logger.info("Received /deadlock request. Initiating deadlock scenario.");
            new Thread(ThreadAnalyzerApplication::acquireDeadlockLock1, "DeadlockThread-1").start();
            new Thread(ThreadAnalyzerApplication::acquireDeadlockLock2, "DeadlockThread-2").start();
            managedThreads.add(new Thread(ThreadAnalyzerApplication::acquireDeadlockLock1, "DeadlockThread-1"));
            managedThreads.add(new Thread(ThreadAnalyzerApplication::acquireDeadlockLock2, "DeadlockThread-2"));
            managedThreads.get(managedThreads.size() - 2).start();
            managedThreads.get(managedThreads.size() - 1).start();
            return "Deadlock scenario initiated!";
        });
    }
}