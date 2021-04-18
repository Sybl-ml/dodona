<template>
  <b-container fluid>
    <b-row>
      <b-col  v-if="checkStatus('Processing')" class="mb-3">
        <h4>Project Is Running ...</h4>
        <b-progress
          :value="value"
          :variant="progressColor"
          height="2rem"
          show-progress
          animated
        ></b-progress>
      </b-col>
      <b-col lg="8" sm="12" v-else-if="checkStatus('Ready')" class="mb-3">
        <h4>Description:</h4>
        <div class="scrollable_description mb-3">
          {{ description }}
        </div>
        <h4>Linked Dataset:</h4>
        <b-button-group size="sm" class="mb-3">
          <b-button variant="secondary" @click="$emit('input-tab')">{{
            dataset_name
          }}</b-button>
          <b-button variant="outline-secondary" v-b-modal.deleteDataCheck
            >X</b-button
          >
        </b-button-group>

        <b-modal
          id="deleteDataCheck"
          ref="deleteDataCheck"
          title="Are your sure?"
          hide-footer
        >
          <p>You are removing {{ dataset_name }} from this project</p>
          <p>Please confirm you are happy to continue</p>
          <b-row class="justify-content-center text-center">
            <b-button class="m-2" variant="success" @click="deleteDataset"
              >Confirm</b-button
            >
            <b-button
              class="m-2"
              variant="warning"
              @click="$bvModal.hide('deleteDataCheck')"
              >Cancel</b-button
            >
          </b-row>
        </b-modal>

        <h4>Job Configuration:</h4>

        <p><b>Job Cost:</b> {{this.jobCost}} credits</p>

        <b-form-group label="Problem Type" label-for="dropdown-form-type">
          <b-form-select
            id="dropdown-form-type"
            size="sm"
            :options="problemTypeOptions"
            v-model="problemType"
          />
        </b-form-group>
        <b-form-group label="Prediction Column" label-for="dropdown-pred-col">
          <b-form-select
            id="dropdown-pred-col"
            size="sm"
            :options="getColumnNames"
            v-model="predColumn"
          />
        </b-form-group>
        <b-button
          v-b-toggle.job-config
          pill
          variant="mid"
          size="sm"
          class="mb-3"
          @click="expandJob = !expandJob"
          >{{ expandJob ? "Advanced" : "Minimize" }}</b-button
        >
        <b-collapse id="job-config">
          <b-form-group
            label="Node Computation Time (mins)"
            label-for="dropdown-form-timeout"
          >
            <b-form-input
              id="dropdown-form-timeout"
              size="sm"
              type="number"
              min="1"
              v-model="nodeComputationTime"
            ></b-form-input>
          </b-form-group>
          <b-form-group
            label="Cluster Size"
            label-for="dropdown-form-cluster-size"
          >
            <b-form-input
              id="dropdown-form-cluster-size"
              size="sm"
              type="number"
              min="1"
              v-model="cluster_size"
            ></b-form-input>
          </b-form-group>
        </b-collapse>
        <h4>To start computation click the button below</h4>
        <div class="text-center">
          <b-button
            @click="start"
            variant="success"
            :disabled="startDisabled"
            size="lg"
          >
            Start <b-icon-play-fill />
          </b-button>
        </div>
      </b-col>
      <b-col lg="8" sm="12" v-else>
        <h4>Description:</h4>
        <div class="scrollable_description mb-3">
          {{ description }}
        </div>
        <h5>To continue you must provide a dataset</h5>
        <file-upload v-model="file" />
        <b-button block size="sm" variant="secondary" @click="processFile"
          >Upload</b-button
        >
      </b-col>
    </b-row>
  </b-container>
</template>

<style scoped>
.input-table {
  overflow-y: scroll;
}

.scrollable_description {
  max-height: 10.5rem;
  overflow: auto;
}
</style>

<script>
import FileUpload from "@/components/FileUpload";

export default {
  name: "ProjectOverview",
  components: {
    FileUpload,
  },
  data() {
    return {
      nodeComputationTime: 10,
      cluster_size: 2,
      value: 64,
      file: null,
      problemType: null,
      predColumn: null,
      expandJob: true,
      problemTypeOptions: [
        {
          value: null,
          text: "Please Choose Classification or Regression",
        },
        {
          value: "classification",
          text: "Classification",
        },
        {
          value: "regression",
          text: "Regression",
        },
      ],
    };
  },
  props: {
    projectId: String,
    description: String,
    status: String,
    dataset_name: String,
    dataset_head: Object,
    dataset_date: Date,
    dataset_types: Object,
    dataset_train_size: Number,
    dataset_predict_size: Number
  },
  computed: {
    getDatasetDate() {
      return `${this.dataset_date.toLocaleString("en-GB", {
        dateStyle: "short",
      })} - ${this.dataset_date.toLocaleString("en-GB", {
        timeStyle: "short",
      })}`;
    },
    progressColor() {
      if (this.value === 100) {
        return "completed";
      } else if (this.value < 25) {
        return "warning";
      } else if (this.value < 50) {
        return "primary";
      } else if (this.value < 75) {
        return "ready";
      }
    },
    getColumnNames() {
      let keys = Object.keys(this.dataset_types);
      let options = [
        {
          value: null,
          text: "Please select the column to predict",
        },
      ];
      keys.forEach((key) => options.push({ value: key, text: key }));
      return options;

    },
    startDisabled() {
      return this.predColumn == null || this.problemType == null || this.jobCost > this.$store.state.user_data.credits;
    },
    jobCost() {
      let size = this.dataset_train_size + this.dataset_predict_size;
      return Math.max(Math.floor(size / 1000), 1) * this.cluster_size * Object.keys(this.dataset_types).length;
    }
  },
  methods: {
    async start() {
      let payload = {
        projectId: this.projectId,
        node_computation_time: this.nodeComputationTime,
        cluster_size: this.cluster_size,
        prediction_type: this.problemType,
        prediction_column: this.predColumn,
      };

      this.$store.dispatch("startProcessing", payload);
    },
    async deleteDataset() {
      console.log(this.projectId)
      this.$store.dispatch("deleteData", this.projectId);
      this.$refs["deleteDataCheck"].hide();
    },
    async processFile() {
      console.log("Processing uploaded data");
      if (this.file.file) {
        try {
          console.log("Processing single file");
          let payload = {
            project_id: this.projectId,
            multipart: false,
            files: this.file.file,
          };
          this.$store.dispatch("sendFile", payload);
        } catch (e) {
          console.warn(e.message);
        }
      } else if (this.file.train) {
        // Use train endpoint and predict endpoint
        console.log("Processing 2 files");
        try {
          let payload = {
            project_id: this.projectId,
            multipart: true,
            files: {
              train: this.file.train,
              predict: this.file.predict,
            },
          };
          this.$store.dispatch("sendFile", payload);
        } catch (e) {
          console.warn(e.message);
        }
      }
    },
    checkStatus(status_check) {
      return this.status == status_check;
    },
  },
};
</script>
