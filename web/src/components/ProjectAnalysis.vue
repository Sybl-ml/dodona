<template>
  <b-container fluid>
    <h4 class="mb-0">Analysis</h4>

    <b-form-group label="Select a Column:" label-for="dropdown-analysis-select">
      <b-form-select
        id="dropdown-analysis-select"
        size="sm"
        :options="getAnalysisOptions"
        v-model="analysis_selected"
      />
    </b-form-group>

    <b-row>
      <div v-if="this.analysis.columns[this.analysis_selected].Numerical">
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
      </div>
    </b-row>
  </b-container>
</template>

<script>
import DataAnalyticsBar from "@/components/charts/DataAnalyticsBar";
import NumericalDataAnalyticsBar from "@/components/charts/NumericalDataAnalyticsBar";

export default {
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
    analysis: Object,
  },
  computed: {
    getAnalysisOptions() {
      let keys = Object.keys(this.analysis.columns);
      this.analysis_selected = keys[0];

      if (this.$refs.analysis_chart)
        this.$refs.analysis_chart.renderNewData(this.analysis.columns[this.analysis_selected].Categorical.values);
      return keys;
    },
    update_analysis() {

    },
  },
};
</script>
