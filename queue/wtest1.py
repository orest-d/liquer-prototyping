# Example of using multiprocessing and SyncManager to create a queue
# that can be shared between processes.

import multiprocessing
import time


def worker(queue):
    for i in range(10):
        print("create item: %d" % i)
        queue.put(i)
        time.sleep(0.1)

def reader(queue):
    while True:
        if not queue.empty():
            item = queue.get()
            print(item)
        else:
            time.sleep(0.2)

def xxx(manager):
    # Create a queue
    queue = manager.Queue()

    # Create a worker process populating the queue

    # Create a process to run the worker
    p1 = multiprocessing.Process(target=worker, args=(queue,))
    p1.start()

    # Create a process to read from the queue

    # Create a process to run the reader
    p2 = multiprocessing.Process(target=reader, args=(queue,))
    p2.start()

    # Wait for the worker to finish
    p1.join()
    p2.join()

if __name__ == '__main__':
    multiprocessing.freeze_support()
    # Create a manager
    manager = multiprocessing.Manager()
    xxx(manager)
