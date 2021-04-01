
import torch
import torch.nn as nn
import torch.nn.functional as F
import torch.optim as optim
import random
import sys
import os
from multiprocessing import Pool

class Net(nn.Module):


    def __init__(self):
        super(Net, self).__init__()
        # 12 input image channels for each piecetype, 32 output channels, 3x3 square convolution
        # Pad after each convolution
        # kernel
        self.conv1 = nn.Conv2d(12, 128, 3, padding = (1, 1))
        self.conv2 = nn.Conv2d(128, 128, 3, padding = (1, 1))
        self.conv3 = nn.Conv2d(128, 128, 3, padding = (1, 1))
        self.conv4 = nn.Conv2d(128, 128, 3, padding = (1, 1))

        # an affine operation: y = Wx + b
        self.fc1 = nn.Linear(128 * 64, 128)  # 6*6 from image dimension
        self.fc2 = nn.Linear(128, 1)

        self.learning_rate = 0.001
        self.device = torch.device("cuda:0")
        self.criterion = nn.MSELoss()
        self.optimizer = optim.SGD(self.parameters(), lr=self.learning_rate, momentum=0.9)

    def forward(self, x):
        x = F.relu(self.conv1(x))
        x = F.relu(self.conv2(x))
        x = F.relu(self.conv3(x))
        x = F.relu(self.conv4(x))
        x = x.view(-1, self.num_flat_features(x))
        x = F.relu(self.fc1(x))
        x = torch.sigmoid(self.fc2(x))
        return x

    def num_flat_features(self, x):
        size = x.size()[1:]  # all dimensions except the batch dimension
        num_features = 1
        for s in size:
            num_features *= s
        return num_features

    def train_file(self, inputs, labels):
        loss = 0.0
        running_loss = 0.0
        batch_size = 100
        stripe_count = int(len(inputs) / batch_size)
        for r in range(1):
            for i in range(stripe_count):
                # i is number of stripes in chunked list of minibatches
                t_inputs = torch.tensor(inputs[i::stripe_count], device=self.device)
                t_labels = torch.tensor(labels[i::stripe_count], device=self.device)

                self.optimizer.zero_grad()
                outputs = self(t_inputs)
                loss = self.criterion(outputs, t_labels)
                loss.backward()
                self.optimizer.step()

                # print statistics
                running_loss += loss.item()

            # print every round
            if True:
                loss = (running_loss/stripe_count)**(1/2)
                print('\tRound {} Avg Linear Loss: {}'.format(r, loss))
                running_loss = 0.0
        return loss


def split(x):
    return x.strip().split(',')

def tensorfy(x):
    return board_to_tensor(x[0])

def create_label(x):
    return [float(x[2])]
