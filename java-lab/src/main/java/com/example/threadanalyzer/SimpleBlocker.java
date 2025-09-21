package com.example.threadanalyzer;

public class SimpleBlocker {

    private static final Object lock = new Object();

    public static void main(String[] args) throws InterruptedException {
        // Thread that acquires and holds the lock
        Thread blockerThread = new Thread(() -> {
            synchronized (lock) {
                System.out.println("BlockerThread acquired lock. Will hold indefinitely.");
                try {
                    Thread.currentThread().join(); // Hold the lock indefinitely
                } catch (InterruptedException e) {
                    Thread.currentThread().interrupt();
                    System.out.println("BlockerThread interrupted.");
                }
            }
        }, "BlockerThread");

        // Thread that attempts to acquire the same lock and gets BLOCKED
        Thread blockedThread = new Thread(() -> {
            System.out.println("BlockedThread attempting to acquire lock...");
            synchronized (lock) { // This thread will block here
                System.out.println("BlockedThread acquired lock (should not happen if BlockerThread is active).");
            }
        }, "BlockedThread");

        blockerThread.start();
        // Give BlockerThread a moment to acquire the lock
        Thread.sleep(1000);
        blockedThread.start();

        // Keep main thread alive
        Thread.currentThread().join();
    }
}