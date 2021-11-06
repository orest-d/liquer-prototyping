<template>
  <v-app>
    <v-navigation-drawer
      v-model="drawer"
      app
    >
      <!--  -->
    </v-navigation-drawer>

    <v-app-bar app>
      <v-app-bar-nav-icon @click="drawer = !drawer"></v-app-bar-nav-icon>

      <v-toolbar-title>Application</v-toolbar-title>
    </v-app-bar>

    <v-main>
      <!--  -->
    </v-main>
    <StatusBar :status="status" :message="message"/>
  </v-app>
</template>

<script>
import StatusBar from './components/StatusBar';

export default {
  name: 'App',

  components: {
    StatusBar
  },

  data: () => ({
    drawer: false,
    status: "ERROR",
    message: "Everything is fine!",
    url_submit_prefix: "/liquer/submit/",
    url_remove_prefix: "/liquer/cache/remove/",
    url_prefix: "/liquer/q/",
  }),
};
</script>
