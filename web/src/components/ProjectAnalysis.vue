<template>
  <b-card class="h-100 shadow">
    <template #header>
      <h4 class="mb-0">Analysis</h4>
    </template>

    <b-form-group label="Select a Column:" label-for="dropdown-analysis-select">
      <b-form-select
        id="dropdown-analysis-select"
        size="sm"
        :options="getAnalysisOptions"
        v-model="analysis_selected"
        v-on:change="update_analysis"
      />
    </b-form-group>
    <b-row v-if="!analysis_loaded" class="justify-content-center text-center">
      <b-spinner />
    </b-row>
    <div v-else>
      <!-- <div v-if="this.analysis.columns[this.analysis_selected].Numerical">
        <numerical-data-analytics-bar
          :chart-data="
            this.analysis.columns[this.analysis_selected].Numerical.values
          "
          :name="this.analysis_selected"
          ref="analysis_chart"
        />
        <p>
          MAX -
          {{ this.analysis.columns[this.analysis_selected].Numerical.max }}
        </p>
        <p>
          MIN -
          {{ this.analysis.columns[this.analysis_selected].Numerical.min }}
        </p>
        <p>
          AVG -
          {{ this.analysis.columns[this.analysis_selected].Numerical.avg }}
        </p>
      </div>
      <div v-else>
        <data-analytics-bar
          :chart-data="
            this.analysis.columns[this.analysis_selected].Categorical.values
          "
          :name="this.analysis_selected"
          ref="analysis_chart"
        />
      </div> -->
    </div>
  </b-card>
</template>

<script>
import DataAnalyticsBar from "@/components/charts/DataAnalyticsBar";
import NumericalDataAnalyticsBar from "@/components/charts/NumericalDataAnalyticsBar";

export default ({
  name: "ProjectAnalysis",
  components: {
    DataAnalyticsBar,
    NumericalDataAnalyticsBar,
  },
  data() {
    return {
      analysis_selected: null,
    };
  },
  props: {
    id: String,
  },
  computed: {
    project(){
        return this.$store.getters.getProject(this.id);
    },
    analysis() {
      return this.project.analysis;
    },
    getAnalysisOptions() {

          console.log(this.project)
        // let keys = Object.keys(this.analysis.columns);
        // this.analysis_selected = keys[0];
        // return keys;
      
      return [];
    },
    update_analysis() {
      if (this.analysis.columns[this.analysis_selected].Categorical)
        this.$refs.analysis_chart.renderNewData(
          this.analysis.columns[this.analysis_selected].Categorical.values
        );
    },
  },
});
</script>
