<template>
  <div>
    <b-container fluid>
      <b-row>
        <b-col xs="12" order-xs="2" lg="3">
          <b-row>
            <b-col class="mb-3">
              <b-form-input placeholder="Search" block />
            </b-col>
          </b-row>
          <b-row class="text-left">
            <b-col>
              <router-link :to="{ name: 'AddProject' }">
                <b-button variant="primary" class="mb-1 add-new" block
                  ><b-row
                    ><b-col class="text-left">Add new project</b-col
                    ><b-col class="ml-auto text-right">
                      <b-icon-plus-circle /></b-col></b-row></b-button
              ></router-link>
              <router-link
                v-for="p in projects"
                :key="p.id"
                :to="{
                  name: `ProjectView`,
                  params: {
                    projectId: p.id,
                  },
                }"
              >
                <b-card class="mb-1" no-body>
                  <b-row no-gutters>
                    <b-col cols="10">
                      <b-card-body :title="p.name">
                        <b-card-text>
                          {{ p.description }}
                        </b-card-text>
                      </b-card-body>
                    </b-col>
                    <b-col
                      class="text-center justify-content-center"
                      align-self="center"
                      ><b-icon-hourglass
                        v-if="p.status == 'Unfinished'"
                        class="h3" />
                      <b-icon-hourglass-top
                        v-else-if="p.status == 'Ready'"
                        style="color: blue"
                        class="h3" />
                      <b-icon-hourglass-split
                        v-else-if="p.status == 'Processing'"
                        animation="fade"
                        variant="primary"
                        class="h3" />
                      <b-icon-hourglass-bottom
                        v-else-if="p.status == 'Completed'"
                        style="color: green"
                        class="h3"
                    /></b-col>
                  </b-row>
                </b-card>
              </router-link>
            </b-col>
          </b-row>
        </b-col>
        <b-col>
          <router-view></router-view>
        </b-col>
      </b-row>
    </b-container>
  </div>
</template>

<style>
.add-new {
  height: 40px;
  font-size: large;
}
</style>
<script>
import axios from "axios";

export default {
  name: "Dashboard",
  data() {
    return {
      projects: [],
    };
  },
  async mounted() {
    let user_id = $cookies.get("token");

    let response = await axios.get(
      `http://localhost:3001/api/projects/u/${user_id}`
    );

    this.projects = response.data.map((x) => {
      let y = {
        ...x,
        id: x._id.$oid,
        user_id: x.user_id.$oid,
        date_created: x.date_created.$date,
      };
      delete y._id;
      return y;
    });
  },
  methods: {},
};
</script>
