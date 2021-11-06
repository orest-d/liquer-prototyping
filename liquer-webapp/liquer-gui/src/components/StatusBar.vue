<template>
    <v-footer padless>
        <v-col cols="1" style="padding:1px;">
            <v-chip v-if="status=='OK'" label color="gray" dark>
            {{status}}
            </v-chip>
            <v-chip v-if="status=='ERROR'" label color="red" dark>
            {{status}}
            </v-chip>
        </v-col>
        <v-col cols="11" style="padding:1px;" >
            <span>{{message}}</span>
        </v-col>
    </v-footer>
</template>

<script>
  export default {
    props: ['status','message'],
    data: () => ({
        drawer: null
    }),
  }
</script>