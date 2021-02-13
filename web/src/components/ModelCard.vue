<template>
  <b-container fluid>
    <b-row class="justify-content-center">
      <b-col xs="12" lg="7">
        <b-card
          class="mb-4 shadow"
          no-body
          v-b-toggle="`collapse-${this.i}`"
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
                <b-card-text v-else-if="data.locked == true">
                  <b-icon-lock-fill style="color: #fbb000"></b-icon-lock-fill>
                  Locked
                </b-card-text>
                <b-card-text v-else-if="data.status == 'NotStarted'">
                  <b-icon-pause-fill style="color: #fbb000"></b-icon-pause-fill>
                  Not Started
                </b-card-text>
                <b-card-text v-else>
                  <b-icon-exclamation-triangle-fill
                    style="color: #ff1700"
                  ></b-icon-exclamation-triangle-fill>
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
    <b-row class="justify-content-center">
      <b-col xs="12" lg="7">
      <b-collapse
        :id="`collapse-${i}`"
        class="mb-4 nodeExpansion"
        @shown="renderChart(i)"
      >
        <b-card class="shadow" v-if="data.locked == true">
          <b-row class="justify-content-center text-center">
            <b-col lg="4" md="8" sm="10" xs="12">
              <br />
              <h1>
                <b-icon-lock-fill style="color: #fbb000"></b-icon-lock-fill>
              </h1>
              <b-card-title>Unlock Model</b-card-title>
              <b-card-text>Please provide your password to confirm</b-card-text>
              <b-form class="mt-5 mb-3" @submit="onSubmit">
                <b-form-input
                  type="password"
                  id="name"
                  class="mb-3"
                  v-model="password"
                ></b-form-input>
                <b-button type="submit" variant="primary" class="mb-3">
                  Unlock
                </b-button>
              </b-form>
            </b-col>
          </b-row>
        </b-card>
        <b-card class="shadow" v-else-if="data.status == 'NotStarted'">
          <b-card-title>Ready to Start</b-card-title>
          <b-card-text>Run your model script to begin!</b-card-text>
        </b-card>
        <b-card 
          class="shadow-sm p-3 mb-5 bg-white rounded"
          style="border-width: 0.15rem" 
          v-else>
            <model-performance
              :data="performance"
              :ref="`model-performance-${i}`"
            />
            <b-card-text>Total Runs: {{ data.times_run }}</b-card-text>
        </b-card>
      </b-collapse>
      </b-col>
    </b-row>
  </b-container>
</template>

<style>
.nodeExpansion {
  width: 100%;
}
</style>

<script>
import ModelPerformance from "@/components/ModelPerformance";

export default {
  name: "ModelCard",
  props: {
    data: Object,
    i: Number,
  },
  data() {
    return {
      test_performance: [{ performance: 0.5 }, { performance: 0.4 }],
      performance: [],
      password: "",
    };
  },
  components: {
    ModelPerformance,
  },
  async mounted() {
    try {
      let data = await this.$http.get(
        `http://localhost:3001/api/clients/m/{this.data._id.$oid}/performance`,
      );
      this.performance = data.data;
    } catch (err) {
      console.log(err);
    }
  },
  computed: {
    status_variant() {
      console.log(this.data._id.$oid);
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
  methods: {
    async onSubmit() {
      let response = await this.$http.post(
        "http://localhost:3001/api/clients/m/unlock",
        {
          id: this.data._id.$oid,
          password: this.password,
        }
      );

      response = response.data;
      console.log(response);
    },
    renderChart(model_id) {
      this.$refs[`model-performance-${model_id}`].show();
    },
  },
};
</script>
