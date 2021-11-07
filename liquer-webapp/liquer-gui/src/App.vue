<template>
  <v-app>
    <v-navigation-drawer v-model="drawer" app>
      <!--  -->
    </v-navigation-drawer>

    <v-app-bar app>
      <v-app-bar-nav-icon @click="drawer = !drawer"></v-app-bar-nav-icon>

      <v-toolbar-title>Application</v-toolbar-title>
    </v-app-bar>

    <v-main>
      <!--  -->
      <Commands :liquer_url="liquer_url" v-on:info-event="info($event)" v-on:error-event="error($event.message,$event.reason)"/>
    </v-main>
    <StatusBar :status="status" :message="message" />
  </v-app>
</template>

<script>
import StatusBar from "./components/StatusBar";
import Commands from "./components/Commands";

export default {
  name: "App",

  components: {
    StatusBar, Commands,
  },

  data: () => ({
    drawer: false,
    status: "ERROR",
    message: "Everything is fine!",
    url_submit_prefix: "/liquer/submit/",
    url_remove_prefix: "/liquer/cache/remove/",
    liquer_url: "http://127.0.0.1:5000/liquer",
    html: "",
  }),
  methods: {
    info: function (txt) {
      console.log("INFO:" + txt);
      this.message = txt;
      this.status = "OK";
    },
    error: function (txt, reason) {
      console.log("ERROR:" + txt, reason);
      this.message = txt;
      this.status = "ERROR";
      if (reason != null && "body" in reason) {
        this.html = reason.body;
      }
    },
  }
};
</script>
