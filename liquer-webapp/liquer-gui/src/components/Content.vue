<template>
  <v-container class="fill-height" fluid>
    <v-row v-if="mode == 'none'">
      <v-col cols="5" />
      <v-col cols="2">
        <v-progress-circular
          :size="150"
          :width="20"
          color="primary"
          indeterminate
        ></v-progress-circular>
      </v-col>
      <v-col cols="5" />
    </v-row>
    <v-row v-if="mode == 'error'">
      <v-col>
        Query {{ metadata.query }} failed.
        <p>{{ metadata.message }}</p>
      </v-col>
    </v-row>
    <v-row v-if="mode == 'invalid'">
      <v-col> Invalid metadata </v-col>
    </v-row>
    <v-row v-if="mode == 'waiting'">
      <v-col>
        Waiting for {{ metadata.query }}.
        <p>{{ metadata.message }}</p>
      </v-col>
    </v-row>
    <div v-if="mode == 'text'">
      {{ data }}
    </div>
  </v-container>
</template>

<script>
export default {
  props: {
    liquer_url: {
      type: String,
      default: "/liquer",
    },
    data: {
      default: null,
    },
    metadata: {
      default: null,
    },
  },
  data: () => ({
    mode: "none",
    data_ready: false,
    url_query_prefix: "/liquer/q/",
    url_submit_prefix: "/liquer/submit/",
    url_remove_prefix: "/liquer/cache/remove/",
    url_cache_meta_prefix: "/liquer/cache/meta/",
  }),
  methods: {
    info(message, reason = null, query = null) {
      this.message_event({ status: "INFO", message, reason, query });
    },
    error(message, reason = null, query = null) {
      this.message_event({ status: "ERROR", message, reason, query });
    },
    message_event(m) {
      this.$emit("message-event", m);
    },
    update() {
      if (this.metadata == null) {
        this.mode = "none";
      }
      if (this.metadata.status == null) {
        this.mode = "invalid";
      }
      if (this.metadata.status == "error") {
        this.mode = "error";
      }
      if (this.metadata.status == "ready") {
        console.log("READY");
        console.log("Query", this.metadata.query);
        console.log("Type", this.metadata.type_identifier);
        var type_actions = {
          generic() {
            this.just_load("text");
          },
          text() {
            this.just_load("text");
          },
        };
        type_actions[this.metadata.type_identifier].bind(this)();
      }
      return "waiting";
    },
    just_load(mode, query = null) {
      if (query == null) {
        query = this.metadata.query;
      }
      console.log("Just load", query);
      this.$http.get(this.url_query_prefix + query).then(
        function (response) {
          this.data = response.body;
          this.mode = mode;
        }.bind(this),
        function (reason) {
          this.error("Failed loading data", reason, query);
        }.bind(this)
      );
    },
  },
  watch: {
    metadata() {
      this.update();
    },
  },
  computed: {
    dataframe_headers: function () {
      if (this.data == null) {
        return [];
      } else {
        var h = [];

        this.data.schema.fields.forEach(function (x) {
          h.push({
            text: x.name,
            value: x.name,
          });
        });
        console.log(h);
        return h;
      }
    },
    dataframe_rows: function () {
      if (this.data == null) {
        return [];
      } else {
        console.log("rows", this.data.data);
        return this.data.data;
      }
    },
  },
};
</script>