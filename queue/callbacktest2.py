import multiprocessing
from multiprocessing.managers import BaseManager, SyncManager
import random
import time


class MyManager(SyncManager):
    pass


class Aaa:
    def __init__(self,aaa=0):
        self.aaa = [aaa]
    def __str__(self):
        return f"Aaa({self.aaa})"
    def __repr__(self):
        return f"Aaa({self.aaa})"
    def set(self,aaa):
        self.aaa[0] = aaa
    def get(self):
        return self.aaa[0]
    

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
        while True:
            job = jq.get_job()
            print(f"Worker {worker_id} got job {job}")
            time.sleep(0.2)
            result = f"Result-{job} ({jq.get_result('Job1')})"
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

    def get_result(self, job):
        return self.results.get(job, None)

MyManager.register('JobQueue', JobQueue)
MyManager.register('Aaa', Aaa)


if __name__ == '__main__':
    with MyManager() as manager:
        jq = manager.JobQueue()
        for i in range(4):
            jq.submit(f"Job{i}")
            time.sleep(0.1)
        for i in range(4):
            print(f"Result {i}: {jq.get_result(f'Job{i}')}")
        time.sleep(0.5)
        print()
        aaa = manager.Aaa()
        print(f"aaa: {aaa}, get: {aaa.get()}")
        aaa.set(1)
        print(f"aaa: {aaa}, get: {aaa.get()}")
        multiprocessing.Process(target=aaa.set, args=(2,)).start()
        time.sleep(0.1)
        print(f"aaa: {aaa}, get: {aaa.get()}")
        def run(aaa,x):
            time.sleep(0.1*random.random())
            print(f"run aaa: {aaa}, get: {aaa.get()}")
            time.sleep(0.1*random.random())
            aaa.set(x)
            time.sleep(0.1*random.random())
            print(f"run set({x}) aaa: {aaa}, get: {aaa.get()}")

        multiprocessing.Process(target=run, args=(aaa,3)).start()
        time.sleep(0.1)
        print(f"aaa: {aaa}, get: {aaa.get()}")
        multiprocessing.Process(target=run, args=(aaa,4)).start()
        multiprocessing.Process(target=run, args=(aaa,5)).start()
        multiprocessing.Process(target=run, args=(aaa,6)).start()
        time.sleep(1)
        print(f"aaa: {aaa}, get: {aaa.get()}")


