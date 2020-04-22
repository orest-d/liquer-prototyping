import marshal
import pickle
from glob import glob
import types

for p in glob("*.marshal"):
    try:
        print(p)
        code, name, defaults, closure = pickle.load(open(p,"rb"))
        bc = marshal.loads(code)
        f = types.FunctionType(bc, globals(), name, defaults, closure)
        f(111)
    except:
        import traceback
        traceback.print_exc()
    print()