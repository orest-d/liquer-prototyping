import Vue from 'vue'
import App from './App.vue'
import vuetify from './plugins/vuetify'
import VueResource from 'vue-resource';
import VueHighlightJS from 'vue-highlight.js';

// Highlight.js languages (All languages)
import 'vue-highlight.js/lib/allLanguages'
import 'highlight.js/styles/default.css';
import VueCodemirror from 'vue-codemirror'
import 'codemirror/lib/codemirror.css'

Vue.use(VueCodemirror);


Vue.use(VueResource);
Vue.use(VueHighlightJS);

Vue.config.productionTip = false

new Vue({
  vuetify,
  render: h => h(App),
  data: {
    status: "OK",
    message: "",
    url_submit_prefix: "/liquer/submit/",
    url_remove_prefix: "/liquer/cache/remove/",
    url_prefix: "/liquer/q/",
  }
}).$mount('#app')
