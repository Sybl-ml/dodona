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
                <b-card class="mb-1" no-body :class="p.status.toLowerCase()">
                  <b-row
                    no-gutters
                    class="ml-2"
                    style="background-color: white"
                  >
                    <b-col>
                      <b-card-body :title="p.name">
                        <b-card-text>
                          <b-icon-play-fill
                            v-if="p.status == 'Unfinished'"
                            style="color: red"
                          />
                          <b-icon-play-fill
                            v-else-if="p.status == 'Ready'"
                            style="color: blue"
                          />
                          <b-icon-hourglass-split
                            v-if="p.status == 'Processing'"
                            animation="fade"
                            variant="primary"
                          />
                          <b-icon-check2-circle
                            v-else-if="p.status == 'Completed'"
                            style="color: green"
                          />
                          {{ p.status }}
                        </b-card-text>
                      </b-card-body>
                    </b-col>
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

.unfinished {
  background-color: red !important;
}
.ready {
  background-color: blue !important;
}
.processing {
  background-color: var(--primary) !important;
}
.completed {
  background-color: green !important;
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
