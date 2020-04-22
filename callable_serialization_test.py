import marshal
import pickle

def function_example(x,y=123):
    print(f"Called function_example {x} {y}")
    return x+y

class CallableExample:
    def __init__(self, y=234):
        self.y=y
    def __call__(self, x):
        print(f"Called CallableExample {x} {self.y}")
        return x+self.y

if __name__ == "__main__":
    for i, f in enumerate([function_example, (lambda x,C=CallableExample:C(456)(x)), CallableExample(45).__call__]):
        print(f(1))
        code = marshal.dumps(f.__code__)
        b = pickle.dumps((code, f.__name__, f.__defaults__, f.__closure__))

        with open(f"{i}.marshal","wb") as f:
            f.write(b)
