<template>
  <b-container fluid v-if="!loadedProject">
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
    <b-overlay :show="project.status == 'Uploading'" rounded="sm">
      <template #overlay>
        <b-row class="justify-content-center">
          <h2>Data Uploading</h2>
        </b-row>
        <b-row class="justify-content-center">
          <h2>Please Do Not Refresh!</h2>
        </b-row>
        <b-row class="justify-content-center">
          <b-spinner variant="primary" style="width: 3rem; height: 3rem" />
        </b-row>
      </template>
      <b-card no-body class="shadow">
        <b-tabs pills card>
          <b-tab title="Overview" active lazy ref="overviewTab">
            <project-overview v-if="project" v-bind="overviewProps" />
          </b-tab>
          <b-tab
            title="Analysis"
            lazy
            ref="analysisTab"
            :disabled="projectUnfinished"
          >
            <project-analysis :id="projectId" v-bind="analysisProps" />
          </b-tab>
          <b-tab
            title="Input"
            lazy
            ref="inputTab"
            :disabled="projectUnfinished"
          >
            <project-input
              :projectId="projectId"
              :key="projectId"
              v-on:input-tab="viewInput"
            />
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
    </b-overlay>
  </b-container>
</template>

<script>
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
    viewInput() {
      this.$refs.inputTab.activate();
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
      return this.project === undefined;
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
        dataset_train_size: Math.round((p.details.train_size + 99) / 100) * 100,
        dataset_predict_size:
          Math.round((p.details.predict_size + 99) / 100) * 100,
        current_job: p.current_job,
        job_stats: p.job_stats,
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
        datasetName: p.details.dataset_name,
      };
    },
    outputProps() {
      let p = this.project;
      return {
        projectId: this.projectId,
        datasetName: p.details.dataset_name,
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
