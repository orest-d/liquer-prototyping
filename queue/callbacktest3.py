import multiprocessing
from multiprocessing.managers import BaseManager, SyncManager
import random
import time


class MyManager(SyncManager):
    pass


class Aaa:
    def __init__(self,aaa="DEFAULT"):
        self.aaa = [aaa]
    def __str__(self):
        return f"Aaa({self.aaa})"
    def __repr__(self):
        return f"Aaa({self.aaa})"
    def set(self,aaa):
        print(f"set {aaa}")
        print(f"  called from {multiprocessing.current_process().name}")
        self.aaa[0] = aaa
    def get(self):
        print(f"get")
        print(f"  called from {multiprocessing.current_process().name}")
        return self.aaa[0]
    

_workers=[]

def worker_process(jq, worker_id):
    while True:
        job = jq.get_job()
        print(f"Worker {worker_id} got job {job}")
        time.sleep(0.2)
        result = f"Result-{job}"
        jq.set(job, result, worker_id)
        print(f"Worker {worker_id} finished job {job}")

def start_workers(jq):
    global _workers
    p1 = multiprocessing.Process(target=worker_process, args=(jq, "p1"))
    p1.start()
    _workers.append(p1)
#    jq.register_worker(p1)
    p2 = multiprocessing.Process(target=worker_process, args=(jq, "p2"))
    p2.start()
#    jq.register_worker(p2)
    _workers.append(p2)

class JobQueue:
    def __init__(self):
        self.queue = multiprocessing.Queue()
        self.results = {}
        self.callbacks = {}
        self.workers = []

    def register_worker(self,w):
        self.workers.append(w)

    def stop_workers(self):
        for w in self.workers:
            w.terminate()

    def submit(self, job):
        self.queue.put(job)

    def get_job(self):
        return self.queue.get()

    def set(self, job, result, worker_id):
        print(f"set {job} {result} {worker_id}")
        print(f"  called from {multiprocessing.current_process().name}")
        self.results[job] = result
        if job in self.callbacks:
            for f, arg in self.callbacks.get(job,[]):
                f(result, arg, worker_id)

    def get(self, job):
        print(f"get {job}")
        print(f"  called from {multiprocessing.current_process().name}")
        #print(self.results)
        return self.results.get(job, None)

#MyManager.register('JobQueue', JobQueue, exposed=('submit', 'get', 'set'))
MyManager.register('JobQueue', JobQueue)
MyManager.register('Aaa', Aaa)


def run(aaa):
    print(f"run {aaa}")
    print(f"  called from {multiprocessing.current_process().name}")
    aaa.set("RUN")

if __name__ == '__main__':
    with MyManager() as manager:
        print("---")
        aaa = manager.Aaa()
        print(f"aaa: {aaa}, get:\n**{aaa.get()}")
        aaa.set("MAIN")
        print(f"aaa: {aaa}, get:\n**{aaa.get()}")
        multiprocessing.Process(target=aaa.set, args=("PROCESS",)).start()
        time.sleep(0.1)
        print(f"aaa: {aaa}, get:\n**{aaa.get()}")
        multiprocessing.Process(target=run, args=(aaa,)).start()
        time.sleep(0.1)
        print(f"aaa: {aaa}, get:\n**{aaa.get()}")
        print("---")
        print("---")
        print("---")

        jq = manager.JobQueue()
        start_workers(jq)
        jq.set("X", "ResultX", "MAIN")
        for i in range(4):
            jq.submit(f"Job{i}")
            time.sleep(0.1)
        print("---")
        time.sleep(0.5)
        print("---")
        for i in range(4):
            print(f"Result {i}: {jq.get(f'Job{i}')}")
        print()
        input("Press Enter to continue...")
        for worker in _workers:
            worker.terminate()