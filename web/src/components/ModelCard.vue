<template>
  <b-container fluid>
    <b-row class="justify-content-center">
      <b-col xs="12" lg="7">
        <b-card
          class="mb-4 shadow"
          no-body
          v-b-toggle="'collapse-' + String(this.i)"
          :border-variant="status_variant"
          style="border-width: 0.15rem"
          onfocus="this.blur();"
        >
          <b-row no-gutter>
            <b-col>
              <b-card-body>
                <b-card-title>
                  {{ data.name }}
                </b-card-title>
                <b-card-text>
                  <b-icon-clock-fill></b-icon-clock-fill>
                  01:22:30
                </b-card-text>
              </b-card-body>
            </b-col>
            <b-col>
              <b-card-body style="text-align: right">
                <b-card-text v-if="data.status == 'Running'">
                  <b-icon-check-circle-fill
                    small
                    style="color: #00bf26"
                  ></b-icon-check-circle-fill>
                  Running
                </b-card-text>
                <b-card-text v-else-if="data.status == 'Stopped'">
                  <b-icon-x-octagon-fill
                    style="color: #ff643d"
                  ></b-icon-x-octagon-fill>
                  Stopped
                </b-card-text>
                <b-card-text v-else-if="data.status == 'NotStarted'">
                  <b-icon-lock-fill style="color: #fbb000"></b-icon-lock-fill>
                  Not Started
                </b-card-text>
                <b-card-text v-else>
                  <b-icon-exclamation-triangle-fill style="color: #ff1700"></b-icon-exclamation-triangle-fill>
                  Error
                </b-card-text>
                <b-card-text>
                  <b-icon-cash-stack></b-icon-cash-stack>
                  £2.25
                </b-card-text>
              </b-card-body>
            </b-col>
          </b-row>
          <b-row no-gutter class="justify-content-center">
            <b-icon-chevron-compact-down
              font-scale="1.5"
            ></b-icon-chevron-compact-down>
          </b-row>
        </b-card>
      </b-col>
    </b-row>
    <b-row>
      <b-collapse :id="'collapse-' + String(this.i)" class="mb-4 nodeExpansion">
        <b-card class="shadow">
          <b-card-title>Model Analysis</b-card-title>
          <b-card-text>Number of Uses: {{data.times_run}}</b-card-text>
        </b-card>
      </b-collapse>
    </b-row>
  </b-container>
</template>

<style>
.nodeExpansion {
  width: 100%;
}
</style>

<script>
export default {
  name: "ModelCard",
  props: {
    data: Object,
    i: Number,
  },
  computed: {
    status_variant() {
      if (this.data.status === "NotStarted") {
        return "primary";
      } else if (this.data.status === "Running") {
        return "completed";
      } else if (this.data.status === "Stopped") {
        return "warning";
      } else {
        return "secondary";
      }
    },
  },
  methods: {},
};
</script>
