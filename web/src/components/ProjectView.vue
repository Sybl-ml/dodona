<template>
  <b-container fluid>
    <b-card
      class="view"
      style="height: 7rem; border: none; box-shadow: none; background: none"
    >
      <h2>{{ project.name }}</h2>
      <p>
        {{ getProjectDate }}
        <b-badge
          pill
          variant="success"
          class="mx-1"
          v-for="tag in project.tags"
          v-bind:key="tag.id"
          >{{ tag }}</b-badge
        >
      </p>
    </b-card>
    <b-card no-body class="shadow">
      <b-tabs pills card>
        <b-tab title="Overview" active lazy ref="overviewTab">
          <project-overview
            v-if="project"
            v-bind="overviewProps"
          />
        </b-tab>
        <b-tab title="Analysis" lazy ref="analysisTab" :disabled="projectUnfinished">
          <project-analysis :id="projectId" v-bind="analysisProps" />
        </b-tab>
        <b-tab title="Input" lazy ref="inputTab" :disabled="projectUnfinished">
          <project-input :projectId="projectId" :key="projectId" 
            v-on:input-tab="viewInput"/>
        </b-tab>
        <b-tab title="Output" lazy :disabled="projectComplete">
          <project-output
            :disabled="!results"
            :projectId="projectId"
            :key="projectId"
          />
        </b-tab>
        <b-tab title="Settings" lazy>
          <project-settings v-if="project" v-bind="settingsProps" />
        </b-tab>
      </b-tabs>
    </b-card>
  </b-container>
</template>

<script>
import Papa from "papaparse";
import ProjectOverview from "@/components/ProjectOverview";
import ProjectAnalysis from "@/components/ProjectAnalysis";
import ProjectInput from "@/components/ProjectInput";
import ProjectOutput from "@/components/ProjectOutput";
import ProjectSettings from "@/components/ProjectSettings";

export default {
  name: "ProjectView",
  data() {
    return {
      training_data: null,
      predict_data: null,
      loading: false,
      results: null,
      predict_data: null,
      results_loading: false,
    };
  },
  props: {
    projectId: String,
  },
  components: {
    ProjectOverview,
    ProjectAnalysis,
    ProjectInput,
    ProjectOutput,
    ProjectSettings,
  },
  watch: {
    $route() {
      this.$refs.overviewTab.activate();
    },
  },
  methods: {
    async fetchData() {
      this.loading = true;

      let train_response = await this.$http.get(
        `api/projects/${this.projectId}/data/train`
      );

      let train = train_response.data;

      this.training_data = Papa.parse(train, { header: true });

      let predict_response = await this.$http.get(
        `api/projects/${this.projectId}/data/predict`
      );

      let predict = predict_response.data;

      this.predict_data = Papa.parse(predict, { header: true });

      this.loading = false;
    },
    async fetchResults() {
      this.results_loading = true;

      let project_predict = await this.$http.get(
        `api/projects/${this.projectId}/data/predict`
      );

      let project_predictions = await this.$http.get(
        `api/projects/${this.projectId}/predictions`
      );

      this.results = project_predictions.data;
      this.predict_data = project_predict.data;
      this.results_loading = false;
    },
    viewInput() {
      this.$refs.inputTab.activate();
      this.fetchData();
    },
  },
  computed: {
    projectUnfinished() {
      return this.project.status == "Unfinished";
    },
    projectComplete() {
      return this.project.status != "Complete";
    },
    loadedProject() {
      return this.project;
    },
    getProjectDate() {
      if (!this.project.name) {
        return "";
      }
      return `${this.project.date_created.toLocaleString("en-GB", {
        dateStyle: "short",
      })} - ${this.project.date_created.toLocaleString("en-GB", {
        timeStyle: "short",
      })}`;
    },
    project() {
      return this.$store.getters.getProject(this.projectId);
    },
    overviewProps() {
      let p = this.project;

      return {
        projectId: this.projectId,
        description: p.description,
        status: p.status,
        dataset_name: p.details.dataset_name,
        dataset_head: p.details.dataset_head,
        dataset_date: p.details.dataset_date,
        dataset_types: p.details.column_types,
        dataset_train_size: Math.round((p.details.train_size+99)/100)*100,
        dataset_predict_size: Math.round((p.details.predict_size+99)/100)*100,
      };
    },
    analysisProps() {
      let p = this.project;
      return {
        projectId: this.projectId,
        analysis: p.analysis,
      };
    },
    inputProps() {
      let p = this.project;
      return {
        projectId: this.projectId,
        dataset_head: p.details.dataset_head,
        training_data: this.training_data,
        predict_data: this.predict_data,
        dataset_name: p.details.dataset_name,
        loading: this.loading,
      };
    },
    outputProps() {
      let p = this.project;
      return {
        projectId: this.projectId,
        results: this.results,
        predict_data: this.predict_data,
        dataset_name: p.details.dataset_name,
        loading: this.results_loading,
      };
    },
    settingsProps() {
      let p = this.project;
      return {
        projectId: this.projectId,
        name: p.name,
        description: p.description,
        tags: p.tags,
      };
    },
  },
};
</script>
