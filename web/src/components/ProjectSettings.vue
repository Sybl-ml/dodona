<template>
  <b-container fluid class="mt-3">
    <h4>Edit your Project</h4>
    <b-form-group label="Project Name" id="name" class="font-weight-bold">
      <b-row no-gutters class="mb-2">
        <b-col lg="3" class="pr-2"
          ><b-form-input id="name" v-model="new_name"></b-form-input
        ></b-col>
        <b-col
          ><b-button variant="primary" @click="updateName"
            >Rename</b-button
          ></b-col
        >
      </b-row>
    </b-form-group>
    <b-form-group
      label="Project Description"
      id="desc"
      class="font-weight-bold"
    >
      <b-form-textarea
        id="desc"
        label="Change Project Description"
        v-model="new_description"
        class="mb-2"
      />
      <b-button variant="primary" @click="updateDescription"
        ><b-icon-pen /> Edit</b-button
      >
    </b-form-group>

    <b-form-group label="Project Name" id="name" class="font-weight-bold">
      <b-row no-gutters class="mb-2">
        <b-col lg="8" class="pr-2"
          ><b-form-tags
            class="mb-3"
            tag-variant="success"
            tag-pills
            remove-on-delete
            v-model="new_tags"
          ></b-form-tags
        ></b-col>
        <b-col
          ><b-button variant="primary" @click="updateTags"
            >Update</b-button
          ></b-col
        >
      </b-row>
    </b-form-group>
    <b-card border-variant="secondary" class="mt-3 shadow">
      <b-form-group
        label="Delete Project"
        description="WARNING: This is permanent all data and analysis will be deleted"
        class="font-weight-bold"
      >
        <b-button id="delete" variant="secondary" v-b-modal.deleteCheck
          >DELETE</b-button
        >

        <b-modal
          id="deleteCheck"
          ref="deleteCheck"
          title="Are your sure?"
          hide-footer
        >
          <p>You are removing this project: {{ name }}</p>
          <p>Please confirm you are happy to continue</p>
          <b-row class="justify-content-center text-center">
            <b-button class="m-2" variant="success" @click="deleteProject"
              >Confirm</b-button
            >
            <b-button
              class="m-2"
              variant="warning"
              @click="$bvModal.hide('deleteCheck')"
              >Cancel</b-button
            >
          </b-row>
        </b-modal>
      </b-form-group>
    </b-card>
  </b-container>
</template>

<style scoped>
.danger-zone {
  border: 1px solid red;
  border-radius: 3px;
}
</style>

<script>
export default {
  name: "ProjectSettings",
  data() {
    return {
      new_name: this.name,
      new_description: this.description,
      new_tags: this.tags,
    };
  },
  props: {
    projectId: String,
    name: String,
    description: String,
    tags: Array,
  },
  methods: {
    async updateName() {
      let payload = {
        field: "name",
        new_data: this.new_name,
        project_id: this.projectId,
      };
      this.$store.dispatch("updateProject", payload);
    },
    async updateDescription() {
      let payload = {
        field: "description",
        new_data: this.new_description,
        project_id: this.projectId,
      };
      this.$store.dispatch("updateProject", payload);
    },
    async updateTags() {
      let payload = {
        field: "tags",
        new_data: this.new_tags,
        project_id: this.projectId,
      };
      this.$store.dispatch("updateProject", payload);
    },
    async deleteProject() {
      this.$store.dispatch("deleteProject", {
        projectId: this.projectId,
      });

      this.$refs["deleteCheck"].hide();
    },
  },
};
</script>
