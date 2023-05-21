import multiprocessing
from multiprocessing import Pool, Manager

class JobQueue:
    def __init__(self, num_workers):
        self.num_workers = num_workers
        self.manager = Manager()
        self.job_queue = self.manager.Queue()
        self.results = []
        self.pool = None

    def start_workers(self):
        self.pool = Pool(self.num_workers, self.worker_process, (self.job_queue, self.results))

    def stop_workers(self):
        self.pool.close()
        self.pool.join()

    def submit_job(self, query):
        future = self.manager.Event()
        self.job_queue.put((query, future))
        return future

    @staticmethod
    def worker_process(job_queue, results):
        while True:
            query, future = job_queue.get()
            if query is None:
                break
            # Perform the job processing here
            # You can replace the time.sleep() call with your actual job processing logic
            import time
            time.sleep(1)
            result = len(query)  # Just an example result
            results.append(result)
            future.set()

# Usage example
if __name__ == "__main__":
    num_workers = 3
    job_queue = JobQueue(num_workers)
    job_queue.start_workers()

    # Submit jobs
    future1 = job_queue.submit_job("job 1")
    future2 = job_queue.submit_job("job 2")
    future3 = job_queue.submit_job("job 3")

    # Wait for jobs to complete
    future1.wait()
    future2.wait()
    future3.wait()

    # Get results
    print(f"Job 1 result: {job_queue.results[0]}")
    print(f"Job 2 result: {job_queue.results[1]}")
    print(f"Job 3 result: {job_queue.results[2]}")

    job_queue.stop_workers()




############# With asyncio

import asyncio

class JobQueue:
    def __init__(self):
        self.queue = asyncio.Queue()
        self.results = {}
        self.futures = {}

    async def worker(self):
        while True:
            query, future = await self.queue.get()
            if query is None:
                break
            # Perform the job processing here
            # You can replace the await asyncio.sleep() call with your actual job processing logic
            await asyncio.sleep(1)
            result = len(query)  # Just an example result
            self.results[query] = result
            future.set_result(result)
            self.queue.task_done()

    def submit_job(self, query):
        future = asyncio.Future()
        self.futures[query] = future
        self.queue.put_nowait((query, future))
        return future

    async def run(self, num_workers):
        workers = [asyncio.create_task(self.worker()) for _ in range(num_workers)]
        await self.queue.join()

        # Stop workers
        for _ in range(num_workers):
            self.queue.put_nowait((None, None))
        await asyncio.gather(*workers)

# Usage example
async def main():
    num_workers = 3
    job_queue = JobQueue()

    # Start the job queue
    asyncio.create_task(job_queue.run(num_workers))

    # Submit jobs and obtain futures
    future1 = job_queue.submit_job("job 1")
    future2 = job_queue.submit_job("job 2")
    future3 = job_queue.submit_job("job 3")

    # Wait for jobs to complete
    await asyncio.gather(future1, future2, future3)

    # Get results from futures
    print(f"Job 1 result: {future1.result()}")
    print(f"Job 2 result: {future2.result()}")
    print(f"Job 3 result: {future3.result()}")

    # Stop the job queue
    await job_queue.queue.join()

asyncio.run(main())
