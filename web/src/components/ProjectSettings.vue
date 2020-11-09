<template>
  <b-container fluid class="mt-3">
    <h4>Edit your Project</h4>
    <b-form-group label="Project Name" id="name" class="font-weight-bold">
      <b-row no-gutters class="mb-2">
        <b-col lg="3" class="pr-2"
          ><b-form-input id="name" v-model="newName"></b-form-input
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
        v-model="newDescription"
        class="mb-2"
      />
      <b-button variant="primary" @click="updateDescription"
        ><b-icon-pen /> Edit</b-button
      >
    </b-form-group>
    <b-card border-variant="secondary" class="mt-3">
      <b-form-group
        label="Delete Project"
        description="WARNING: This is permanent all data and analysis will be deleted"
        class="font-weight-bold"
      >
        <b-button id="delete" variant="secondary">DELETE</b-button>
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
      newName: this.name,
      newDescription: this.description,
    };
  },
  props: {
    projectId: String,
    name: String,
    description: String,
  },
  methods: {
    updateName() {
      this.$emit("update:name", this.newName);
    },
    updateDescription() {
      this.$emit("update:description", this.newDescription);
    },
    deleteProject() {},
  },
};
</script>
