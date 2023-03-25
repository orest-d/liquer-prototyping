from pyparsing import *
from liquer.parser import Position

def indent(text,prefix="  "):
    return "\n".join(f"{prefix}{x}" for x in text.split("\n"))

class EmptyElement(object):
    def __init__(self, position=None):
        self.position = position or Position()
    @classmethod
    def from_toks(cls, toks):
        return EmptyElement()
    @classmethod
    def from_parser(cls, parser):
        def _action(s, loc, toks):
            position = Position.from_loc(loc, s)
            x = cls.from_toks(toks)
            x.position = position
            return x
        return parser.setParseAction(_action).setName(cls.__name__)

    def encode(self):
        return f"[{self.__class__.__name__}]"

    def render(self, engine):
        return self.encode()

    def __str__(self):
        return self.encode()

    def __repr__(self):
        return f"{self.__class__.__name__}({repr(self.position)})"


class ArrayElement(EmptyElement):
    def __init__(self, array, position=None, name=None):
        self.array = array
        self.position = position or Position()
        self.name=name

    @classmethod
    def from_toks(cls, toks):
        return cls(toks)

    def encode(self):
        if self.name is not None:
            return " ".join(str(x) for x in self.array)
        else:
            content=", ".join(str(x) for x in self.array)
            return f"{self.name}[{content}]"

    def render(self, engine):
        return self.encode()

    def __str__(self):
        return self.encode()

    def __repr__(self):
        ra = ",\n".join(indent(repr(x)) for x in self.array)
        if self.name is None:
            return f"{self.__class__.__name__}([\n{ra}\n],{repr(self.position)})"
        else:
            return f"{self.name}([\n{ra}\n],{repr(self.position)})"

class Text(EmptyElement):
    """Text element in the template AST"""

    def __init__(self, text, position=None):
        self.text = text
        self.position = position or Position()

    @classmethod
    def from_toks(cls, toks):
        return cls("".join(str(x) for x in toks))

    def encode(self):
        return self.text

    def render(self, engine):
        return self.encode()

    def __str__(self):
        return self.text

    def __repr__(self):
        return f"{self.__class__.__name__}({repr(self.text)}, {repr(self.position)})"

def aa(p, name):
    def _action(s, loc, toks):
        position = Position.from_loc(loc, s)
        return ArrayElement(toks, position=position,name=name)
    return (p).setParseAction(_action).setName(name)



text_element = Text.from_parser(Regex(r"[a-zA-Z0-9,;:\(\).?!]+").leaveWhitespace())
whitespace_element = Text.from_parser(Regex(r"[ \t]+").leaveWhitespace())
nextline = aa(Regex(r"\\(\r|\n)").leaveWhitespace(),"NL")
plain_text = aa(OneOrMore(text_element|whitespace_element|nextline).leaveWhitespace(),"Plain")
emphasis= aa(Literal("*") + plain_text + Literal("*"),"EMPH")
bold= aa(Literal("**") + plain_text + Literal("**"),"BOLD")
code= aa(Literal("```") + plain_text + Literal("```"),"CODE")
link_text = Text.from_parser(Literal("[").suppress() + Regex(r"[^\n^\r^\]]*").leaveWhitespace() + Literal("]").suppress())
link1 = aa((link_text + Literal("(")
        + Regex(r"[^\n^\r^\]^\(^\)^#]+[^\n^\r^\(^\)]*")
        + Literal(")")
), "LINK1")
link2 = aa(Regex(r"https?://[^ ^\t^\(^\)^\n]+"),"LINK2")
local_reference = aa((link_text + Literal("(").suppress()
        + Regex(r"#[^\n^\r^\(^\)]+")
        + Literal(")").suppress()
), "LOCAL_REF")


rich_text = aa(OneOrMore(text_element|whitespace_element|nextline|emphasis|bold|code|link1|link2|local_reference),"RichText")
text_line_start = Optional(whitespace_element)+(text_element|link1|link2|local_reference)
text_line = aa(text_line_start + ZeroOrMore(rich_text) + LineEnd(),"Line")
text_line_l1 = aa(Literal("  ")+text_line,"TXT1")
text_line_l2 = aa(Literal("    ")+text_line,"TXT2")
text_line_l3 = aa(Literal("      ")+text_line,"TXT3")

star_list_element_l1 = aa(Literal("* ")+text_line.leaveWhitespace()+ZeroOrMore(text_line_l1.leaveWhitespace()).leaveWhitespace(),"SLE1")
dash_list_element_l1 = aa(Literal("- ")+text_line.leaveWhitespace()+ZeroOrMore(text_line_l1.leaveWhitespace()).leaveWhitespace(),"DLE1")
star_list_element_l2 = aa(Literal("  * ")+text_line+ZeroOrMore(text_line_l2),"SLE2")
dash_list_element_l2 = aa(Literal("  - ")+text_line+ZeroOrMore(text_line_l2),"DLE2")
star_list_element_l3 = aa(Literal("    * ")+text_line+ZeroOrMore(text_line_l3),"SLE3")
dash_list_element_l3 = aa(Literal("    - ")+text_line+ZeroOrMore(text_line_l3),"DLE3")

list_l3 = aa(OneOrMore(star_list_element_l3)|OneOrMore(dash_list_element_l3),"L3")

star_list_item_l2 = aa(star_list_element_l2 + Optional(list_l3),"SLI2")
dash_list_item_l2 = aa(dash_list_element_l2 + Optional(list_l3),"DLI2")
list_l2 = aa((OneOrMore(star_list_item_l2))|(OneOrMore(dash_list_item_l2)),"L2")

star_list_item_l1 = aa(star_list_element_l1 + Optional(list_l2),"SLI1")
dash_list_item_l1 = aa(dash_list_element_l1 + Optional(list_l2),"DLI1")
list_l1 = aa((OneOrMore(star_list_item_l1))|(OneOrMore(dash_list_item_l1)),"L1")

identifier = aa(Regex(r"[a-zA-Z0-9_]+"),"ID")
code_element=Regex(r"(`[^`]+)|(``[^`]+)|([^`]+)")
code_block = aa((
    Literal("```")+Optional(identifier)+LineEnd()+
    ZeroOrMore(code_element)+
    Literal("```")
    ).leaveWhitespace(),"CODEBLOCK")

#print(emphasis.parseString("*Hello, world!*"))
#print(text_line.parseString("*Hello, world!*"))
#print(rich_text.parseString("Hello, *world*!"))
#print(text_line.parseString("Hello, *world*!"))
#print(text_line.parseString("Hello, world!\\\nxxx *yyy dsgfg* dgsdfg **dgag**"))
#print(OneOrMore(text_line).parseString("Hello, *world*!\nfgaga"))
#print(repr(list_l1.leaveWhitespace().parseString("""* Hello, *world*!
#* Hello?
#  world?
#* ZZZ""")))

print(repr(list_l1.leaveWhitespace().parseString("""* Hello
* Hello?
  world?
  * aaa
  * bbb
* ccc
  - ddd
""")))

print(repr(code_block.leaveWhitespace().parseString("""```xx
some

code
```""")))
print(text_line.parseString("  [Hello](http://example.com), *world*! [anchor](#anchor)"))
