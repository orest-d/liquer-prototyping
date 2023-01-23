<template>
  <v-container>
      <v-row><v-col>HELLO HIGHLIGHT</v-col></v-row>
    <v-row class="text-center">
      <v-col cols="6">
        <prism-editor
          class="my-editor"
          v-model="code"
          :highlight="highlighter"
          :line-numbers="lineNumbers"
        ></prism-editor>
      </v-col>
      <v-col cols="6">
        <div>{{ code }}</div>
      </v-col>
    </v-row>
  </v-container>
</template>

<script>
import { PrismEditor } from "vue-prism-editor";
import "vue-prism-editor/dist/prismeditor.min.css"; // import the styles somewhere

// import highlighting library (you can use any library you want just return html string)
import { highlight, languages} from "prismjs/components/prism-core";
//import { loadLanguages } from "prismjs/components";
import "prismjs/components/prism-clike";
import "prismjs/components/prism-yaml";
import "prismjs/components/prism-javascript";
import "prismjs/components/prism-python";
import "prismjs/themes/prism-tomorrow.css"; // import syntax highlighting styles


export default {
  components: {
    PrismEditor
  },
   name: "HelloWorld",

  data: () => ({
      lineNumbers:true,
    code: "print('hello')",
  }),
  methods: {
    highlighter(code) {
      return highlight(code, languages.python); //returns html
    }
  },
  created(){
//    loadLanguages(["python", "yaml", "sql", "html", "css", "javascript"]);
  }
};
</script>
<style lang="scss">
// required class
.my-editor {
  background: #2d2d2d;
  color: #ccc;

  font-family: Fira code, Fira Mono, Consolas, Menlo, Courier, monospace;
  font-size: 14px;
  line-height: 1.5;
  padding: 5px;
}

// optional
.prism-editor__textarea:focus {
  outline: none;
}
</style>
