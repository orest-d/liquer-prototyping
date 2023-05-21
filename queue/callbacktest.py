import multiprocessing
from multiprocessing.managers import BaseManager, SyncManager
import time


class MyManager(SyncManager):
    pass


class JobQueue:
    def __init__(self):
        self.queue = multiprocessing.Queue()
        self.results = {}
        self.callbacks = {}
        self.p1 = multiprocessing.Process(target=self.worker_process, args=(self, "p1"))
        self.p1.start()
        self.p2 = multiprocessing.Process(target=self.worker_process, args=(self, "p2"))
        self.p2.start()

    @staticmethod
    def worker_process(jq, worker_id):
        def worker_callback(result, arg, worker_id):
            print(f"Worker {worker_id}/{arg} got result {result}")
        time.sleep(0.5)
        jq.add_callback("Job1", worker_callback, f"WorkerCallback1-{worker_id}")
        jq.add_callback("Job2", worker_callback, f"WorkerCallback2-{worker_id}")
        jq.add_callback("Job3", worker_callback, f"WorkerCallback3-{worker_id}")
        jq.add_callback("Job4", worker_callback, f"WorkerCallback4-{worker_id}")
        while True:
            job = jq.get_job()
            print(f"Worker {worker_id} got job {job}")
            time.sleep(0.2)
            result = f"Result-{job}"
            jq.set_result(job, result, worker_id)
            print(f"Worker {worker_id} finished job {job}")

    def submit(self, job):
        self.queue.put(job)

    def get_job(self):
        return self.queue.get()

    def set_result(self, job, result, worker_id):
        self.results[job] = result
        if job in self.callbacks:
            for f, arg in self.callbacks.get(job,[]):
                f(result, arg, worker_id)

    def add_callback(self, job, callback, arg):
        if job not in self.callbacks:
            self.callbacks[job] = []
        print(f"Adding callback {len(self.callbacks)}:{len(self.callbacks[job])} for job {job}/{arg}")
        self.callbacks[job].append((callback,arg))

MyManager.register('JobQueue', JobQueue)

if __name__ == '__main__':
    def server_callback(result, arg, worker_id):
        print(f"Server {worker_id}/{arg} got result {result}")
    with MyManager() as manager:
        jq = manager.JobQueue()
        jq.results = manager.dict()
        jq.callbacks = manager.dict()
        jq.add_callback("Job1", server_callback, f"ServerCallback1")
        jq.add_callback("Job2", server_callback, f"ServerCallback2")
        jq.add_callback("Job3", server_callback, f"ServerCallback3")
        jq.add_callback("Job4", server_callback, f"ServerCallback4")
        for i in range(10):
            jq.submit(f"Job{i}")
            time.sleep(0.1)
